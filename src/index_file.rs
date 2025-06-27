use crate::index_builder::{FileContent, FileIndex, FileLine, Index, LineIndex, NgramIndex};
use bincode::{self, Decode, Encode};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};

#[derive(Debug, Decode, Encode)]
pub struct Range {
    pub start: usize,
    pub len: usize,
}

#[derive(Debug, Decode, Encode)]
pub struct FileLineRange(pub Range);

#[derive(Debug, Decode, Encode)]
pub struct AbsPathRange(pub Range);

#[derive(Debug, Decode, Encode)]
pub struct NgramRange(pub Range);

pub struct Data {
    file_paths: Vec<(FileIndex, FilePathData)>,
    file_lines: Vec<(FileLine, FileLineData)>,
    ngrams: Vec<(NgramIndex, NgramData)>,
    bincode_config: bincode::config::Configuration,
}
#[derive(Decode, Encode)]
pub struct FileLineData(String);

#[derive(Decode, Encode)]
pub struct FilePathData(String);

#[derive(Decode, Encode)]
pub struct NgramData(Vec<FileLine>);

#[derive(Decode, Encode)]
pub struct IndexData {
    id_to_file: HashMap<FileIndex, AbsPathRange>,
    file_line: HashMap<FileLine, FileLineRange>,
    ngram_to_file_line: HashMap<NgramIndex, NgramRange>,
}

impl Data {
    pub(crate) fn new(index: Index) -> Data {
        let (file_paths, content): (Vec<_>, Vec<_>) = index
            .id_to_file
            .into_iter()
            .map(|(file_index, file_content): (FileIndex, FileContent)| {
                (
                    (
                        file_index.clone(),
                        FilePathData(file_content.get_name().path().to_string()),
                    ),
                    (file_index, file_content),
                )
            })
            .unzip();
        let file_lines = content
            .into_iter()
            .map(|(file_index, file)| {
                file.lines()
                    .into_iter()
                    .enumerate()
                    .map(|(id, line)| (id + 1, line))
                    .map(|(line_id, line)| {
                        let file_line =
                            FileLine::new(file_index.clone(), LineIndex::new(line_id as u32));
                        (file_line, FileLineData(line.into()))
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<(FileLine, FileLineData)>>();
        let ngrams = index
            .ngram_to_file_line
            .into_iter()
            .map(|(ngram_index, filelines)| (ngram_index, NgramData(filelines)))
            .collect();
        Self {
            file_paths,
            file_lines,
            ngrams,
            bincode_config: bincode::config::standard(),
        }
    }

    pub fn dump(&self) -> Result<(), io::Error> {
        let mut index_data = IndexData::new();
        let mut output = fs::File::create("igrep.dat")?;
        let offset = 0_usize;

        let offset = self.dump_id_to_file(&mut index_data, &mut output, offset)?;
        let offset = self.dump_file_lines(&mut index_data, &mut output, offset)?;
        let _offset = self.dump_ngrams(&mut index_data, &mut output, offset)?;

        index_data.dump()?;
        Ok(())
    }

    fn dump_id_to_file(
        &self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: usize,
    ) -> Result<usize, io::Error> {
        self.file_paths
            .iter()
            .map(|(file_index, file_path_data)| {
                bincode::encode_to_vec(file_path_data, self.bincode_config.clone())
                    .and_then(|data| Ok((file_index.clone(), data)))
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to encode file path data: {}", e),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (file_index, data)| {
                let len = data.len();
                let range = AbsPathRange(Range::new(offset, len));
                output.write_all(&data)?;
                index_data.add_file(file_index.clone(), range).map_or_else(
                    || Ok(len + offset),
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
        &self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: usize,
    ) -> Result<usize, io::Error> {
        self.file_lines
            .iter()
            .map(|(file_line, file_data)| {
                bincode::encode_to_vec(file_data, self.bincode_config.clone())
                    .and_then(|data| Ok((file_line.clone(), data)))
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to encode file line data: {}", e),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (file_line, data)| {
                let len = data.len();
                let range = FileLineRange(Range::new(offset, len));
                output.write_all(&data)?;
                index_data.add_file_line(file_line, range).map_or_else(
                    || Ok(len + offset),
                    |_| {
                        Err(io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            format!("File line already exists in index"),
                        ))
                    },
                )
            })
    }

    fn dump_ngrams(
        &self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: usize,
    ) -> Result<usize, io::Error> {
        self.ngrams
            .iter()
            .map(|(ngram_index, file_lines)| {
                bincode::encode_to_vec(file_lines, self.bincode_config.clone())
                    .and_then(|data| Ok((ngram_index.clone(), data)))
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to encode ngram data: {}", e),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .try_fold(offset, |offset, (ngram_index, data)| {
                let len = data.len();
                let range = NgramRange(Range::new(offset, len));
                output.write_all(&data)?;
                index_data
                    .add_ngram(ngram_index.clone(), range)
                    .map_or_else(
                        || Ok(len + offset),
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

impl IndexData {
    pub(crate) fn new() -> Self {
        IndexData {
            id_to_file: HashMap::new(),
            file_line: HashMap::new(),
            ngram_to_file_line: HashMap::new(),
        }
    }

    pub fn from_data(data: Vec<u8>) -> Result<Self, io::Error> {
        bincode::decode_from_slice(&data, bincode::config::standard())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to decode index data: {}", e),
                )
            })
            .map(|(index_data, _)| index_data)
    }

    pub fn get_ngram_range(&self, ngram_index: &NgramIndex) -> Option<&NgramRange> {
        self.ngram_to_file_line.get(ngram_index)
    }
    pub(crate) fn add_file_line(
        &mut self,
        file_line: FileLine,
        range: FileLineRange,
    ) -> Option<FileLineRange> {
        self.file_line.insert(file_line, range)
    }

    pub(crate) fn add_file(
        &mut self,
        file_index: FileIndex,
        range: AbsPathRange,
    ) -> Option<AbsPathRange> {
        self.id_to_file.insert(file_index, range)
    }

    pub(crate) fn add_ngram(
        &mut self,
        ngram_index: NgramIndex,
        range: NgramRange,
    ) -> Option<NgramRange> {
        self.ngram_to_file_line.insert(ngram_index, range)
    }

    pub(crate) fn dump(&self) -> Result<(), io::Error> {
        let mut output = fs::File::create("igrep.idx")?;
        let encoded = bincode::encode_to_vec(self, bincode::config::standard()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to encode index data: {}", e),
            )
        })?;
        output.write_all(&encoded)?;
        Ok(())
    }

    pub fn show_info(&self) {
        println!("Index contains:");
        println!(
            "  {} files {}",
            self.id_to_file.len(),
            std::mem::size_of::<AbsPathRange>()
        );
        println!(
            "  {} file lines {}",
            self.file_line.len(),
            std::mem::size_of::<FileLineRange>()
        );
        println!(
            "  {} ngrams {}",
            self.ngram_to_file_line.len(),
            std::mem::size_of::<NgramRange>()
        );
    }
}

impl Range {
    pub fn new(start: usize, len: usize) -> Self {
        Range { start, len }
    }
}

pub trait FromData {
    fn from_data(data: Vec<u8>) -> Result<Self, io::Error>
    where
        Self: Decode<()> + Encode,
    {
        bincode::decode_from_slice(&data, bincode::config::standard())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to decode index data: {}", e),
                )
            })
            .map(|(index_data, _)| index_data)
    }
}

impl NgramData for FromData{
    
}