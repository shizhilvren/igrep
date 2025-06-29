use crate::index_builder::{FileContent, FileIndex, FileLineIndex, Index, LineIndex, NgramIndex};
use bincode::{self, Decode, Encode};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone, Copy)]
pub struct Range {
    #[wasm_bindgen(readonly)]
    pub start: usize,
    #[wasm_bindgen(readonly)]
    pub len: usize,
}

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone)]
pub struct FileLineRange(pub Range);

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone)]
pub struct FileRange(pub Range);

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone, Copy)]
pub struct NgramRange(pub Range);

pub struct Data {
    file_lines: Vec<(FileLineIndex, FileLineData)>,
    file_paths: HashMap<FileIndex, FileData>,
    ngrams: Vec<(NgramIndex, NgramData)>,
}
#[derive(Decode, Encode)]
pub struct FileLineData(String);

#[derive(Decode, Encode)]
pub struct FileData {
    name: String,
    lines_range: HashMap<LineIndex, FileLineRange>,
}

#[derive(Decode, Encode, Debug)]
pub struct NgramData(Vec<FileLineIndex>);

#[wasm_bindgen]
#[derive(Decode, Encode)]
pub struct IndexData {
    id_to_file: HashMap<FileIndex, FileRange>,
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
                        FileData::new(file_content.get_name().path().to_string()),
                    ),
                    (file_index, file_content),
                )
            })
            .unzip();
        let file_paths = file_paths.into_iter().collect::<HashMap<_, _>>();
        let file_lines = content
            .into_iter()
            .map(|(file_index, file)| {
                file.lines()
                    .into_iter()
                    .enumerate()
                    .map(|(id, line)| (id + 1, line))
                    .map(|(line_id, line)| {
                        let file_line =
                            FileLineIndex::new(file_index.clone(), LineIndex::new(line_id as u32));
                        (file_line, FileLineData(line.into()))
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<(FileLineIndex, FileLineData)>>();
        let ngrams = index
            .ngram_to_file_line
            .into_iter()
            .map(|(ngram_index, filelines)| (ngram_index, NgramData(filelines)))
            .collect();
        Self {
            file_paths,
            file_lines,
            ngrams,
        }
    }

    pub fn dump(&mut self) -> Result<(), io::Error> {
        let mut index_data = IndexData::new();
        let mut output = fs::File::create("igrep.dat")?;
        let offset = 0_usize;

        let offset = self.dump_file_lines(&mut index_data, &mut output, offset)?;
        let offset = self.dump_id_to_file(&mut index_data, &mut output, offset)?;
        let _offset = self.dump_ngrams(&mut index_data, &mut output, offset)?;

        index_data.dump()?;
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
        offset: usize,
    ) -> Result<usize, io::Error> {
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
                let range = FileRange(Range::new(offset, len));
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
        &mut self,
        index_data: &mut IndexData,
        output: &mut fs::File,
        offset: usize,
    ) -> Result<usize, io::Error> {
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
                let range = FileLineRange(Range::new(offset, len));
                output.write_all(&data)?;
                self.insert_file_lines_range(&file_line, range);
                Ok(len + offset)
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
                file_lines
                    .to_data()
                    .and_then(|data| Ok((ngram_index.clone(), data)))
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

#[wasm_bindgen]
impl IndexData {
    pub(crate) fn new() -> Self {
        IndexData {
            id_to_file: HashMap::new(),
            ngram_to_file_line: HashMap::new(),
        }
    }

    pub fn get_ngram_range(&self, ngram_index: &NgramIndex) -> Option<NgramRange> {
        self.ngram_to_file_line.get(ngram_index).cloned()
    }

    pub fn get_file_range(&self, file_index: &FileIndex) -> Option<FileRange> {
        self.id_to_file.get(file_index).cloned()
    }

    pub(crate) fn add_file(
        &mut self,
        file_index: FileIndex,
        range: FileRange,
    ) -> Option<FileRange> {
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
            std::mem::size_of::<FileRange>()
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

impl NgramData {
    pub fn file_lines(&self) -> &Vec<FileLineIndex> {
        &self.0
    }
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl FileLineData {
    pub fn get(&self) -> &String {
        &self.0
    }
}

impl FileData {
    pub fn new(name: String) -> Self {
        FileData {
            name,
            lines_range: HashMap::new(),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn lines_range(&self, line_index: &LineIndex) -> Option<&FileLineRange> {
        self.lines_range.get(line_index)
    }

    pub fn insert_line_range(&mut self, line_index: LineIndex, range: FileLineRange) {
        assert!(!self.lines_range.contains_key(&line_index));
        self.lines_range.entry(line_index).or_insert(range);
    }
}

pub trait FromToData {
    fn from_data(data: Vec<u8>) -> Result<Self, io::Error>
    where
        Self: Decode<()>,
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
    fn to_data(&self) -> Result<Vec<u8>, io::Error>
    where
        Self: Encode,
    {
        bincode::encode_to_vec(self, bincode::config::standard()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to encode index data: {}", e),
            )
        })
    }
}

impl FromToData for IndexData {}

impl FromToData for NgramData {}
impl FromToData for NgramRange {}
impl FromToData for NgramIndex {}

impl FromToData for FileLineData {}
impl FromToData for FileLineRange {}
impl FromToData for FileLineIndex {}

impl FromToData for FileIndex {}
impl FromToData for FileRange {}
impl FromToData for FileData {}
