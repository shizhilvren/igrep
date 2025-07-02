use crate::builder::FileContent;
use crate::data::{FileData, FileLineData, FromToData, IndexData, NgramData};
use crate::index::{FileIndex, FileLineIndex, LineIndex, NgramIndex};
use crate::range::{FileLineRange, FileRange, NgramRange, Range};
use bincode::{self, Decode as bincode_decode, Encode as bincode_encode};
use std::io::Write;
use std::path;
use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    io::{self, Error},
};

pub type Offset = u64;
pub struct Encode {
    file_lines: Vec<(FileLineIndex, FileLineData)>,
    file_paths: HashMap<FileIndex, FileData>,
    ngrams: Vec<(NgramIndex, NgramData)>,
}

struct EncodeHelp {
    fid_to_file: Vec<(FileIndex, FileContent)>,
    ngram_to_file_line: HashMap<NgramIndex, Vec<FileLineIndex>>,
}

impl EncodeHelp {
    pub fn file_paths(&self) -> HashMap<FileIndex, FileData> {
        self.fid_to_file
            .iter()
            .map(|(fid, file_content)| {
                (
                    fid.clone(),
                    FileData::new(file_content.get_name().path().to_string()),
                )
            })
            .collect::<HashMap<_, _>>()
    }
    pub fn file_lines(&self) -> Vec<(FileLineIndex, FileLineData)> {
        self.fid_to_file
            .iter()
            .map(|(fid, file_content)| {
                file_content
                    .lines()
                    .iter()
                    .enumerate()
                    .map(|(id, line)| (id + 1, line))
                    .map(|(id, line)| {
                        (
                            FileLineIndex::new(fid.clone(), LineIndex::new(id as u32)),
                            FileLineData::new(line.clone()),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn ngrams(&self) -> Vec<(NgramIndex, NgramData)> {
        self.ngram_to_file_line
            .iter()
            .map(|(nid, fid_lids)| (nid.clone(), NgramData::new(fid_lids)))
            .collect::<Vec<_>>()
    }
}

impl Encode {
    pub(crate) fn new(
        fid_to_file: Vec<(FileIndex, FileContent)>,
        ngram_to_file_line: HashMap<NgramIndex, Vec<FileLineIndex>>,
    ) -> Self {
        let encode_help = EncodeHelp {
            fid_to_file,
            ngram_to_file_line,
        };

        Self {
            file_paths: encode_help.file_paths(),
            file_lines: encode_help.file_lines(),
            ngrams: encode_help.ngrams(),
        }
    }

    pub fn dump(&mut self, path: &path::Path) -> Result<(), io::Error> {
        let mut index_data = IndexData::new();
        let igrep_data = path.join("igrep.dat");
        let mut output = fs::File::create(igrep_data)?;
        let offset = 0_u64;

        let offset = self.dump_file_lines(&mut output, offset)?;
        let offset = self.dump_id_to_file(&mut index_data, &mut output, offset)?;
        let _offset = self.dump_ngrams(&mut index_data, &mut output, offset)?;

        index_data.dump(path)?;
        Ok(())
    }

    fn insert_file_lines_range(
        &mut self,
        file_line_index: &FileLineIndex,
        file_line_range: FileLineRange,
    ) {
        assert!(self.file_paths.contains_key(file_line_index.file_id()));
        self.file_paths
            .entry(file_line_index.file_id().clone())
            .and_modify(|file_data| {
                file_data.insert_line_range(file_line_index.line_id().clone(), file_line_range);
            });
    }

    fn dump_id_to_file(
        &self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: Offset,
    ) -> Result<Offset, io::Error> {
        self.file_paths
            .iter()
            .map(|(file_index, file_path_data)| {
                file_path_data
                    .to_data()
                    .and_then(|data| Ok((file_index.clone(), data)))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (file_index, data)| {
                let len = data.len();
                let range = FileRange(Range::new(offset, len as u32));
                output.write_all(&data)?;
                index_data.add_file(file_index.clone(), range).map_or_else(
                    || Ok(len as u64 + offset),
                    |_| {
                        Err(io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            format!("File index already exists in index"),
                        ))
                    },
                )
            })
    }

    fn dump_file_lines(
        &mut self,
        output: &mut fs::File,
        offset: Offset,
    ) -> Result<Offset, io::Error> {
        self.file_lines
            .iter()
            .map(|(file_line, file_data)| {
                file_data
                    .to_data()
                    .and_then(|data| Ok((file_line.clone(), data)))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (file_line, data)| {
                let len = data.len();
                let range = FileLineRange(Range::new(offset, len as u32));
                output.write_all(&data)?;
                self.insert_file_lines_range(&file_line, range);
                Ok(len as u64 + offset)
            })
    }

    fn dump_ngrams(
        &self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: Offset,
    ) -> Result<Offset, io::Error> {
        self.ngrams
            .iter()
            .map(|(ngram_index, file_lines)| {
                file_lines
                    .to_data()
                    .and_then(|data| Ok((ngram_index.clone(), data)))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (ngram_index, data)| {
                let len = data.len();
                let range = NgramRange(Range::new(offset, len as u32));
                output.write_all(&data)?;
                index_data
                    .add_ngram(ngram_index.clone(), range)
                    .map_or_else(
                        || Ok(len as u64 + offset),
                        |_| {
                            Err(io::Error::new(
                                io::ErrorKind::AlreadyExists,
                                format!("Ngram index already exists in index"),
                            ))
                        },
                    )
            })
    }
}
