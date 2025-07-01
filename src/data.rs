use crate::index::{FileIndex, FileLineIndex, LineIndex, NgramIndex};
use crate::range::{FileLineRange, FileRange, NgramRange, Range};
use bincode::{self, Decode, Encode};
use std::io::Write;
use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    io::{self, Error},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Decode, Encode, Debug)]
pub struct IndexData {
    id_to_file: HashMap<FileIndex, FileRange>,
    ngram_to_file_line: HashMap<NgramIndex, NgramRange>,
}

#[derive(Decode, Encode)]
pub struct FileLineData(String);

#[derive(Decode, Encode)]
pub struct FileData {
    name: String,
    lines_range: HashMap<LineIndex, FileLineRange>,
}

#[derive(Decode, Encode, Debug)]
pub struct NgramData {
    file_lines: Vec<(FileIndex, Vec<LineIndex>)>,
}

impl NgramData {
    pub fn file_lines(&self) -> Vec<FileLineIndex> {
        self.file_lines
            .iter()
            .flat_map(|(fid, lid)| {
                lid.clone()
                    .into_iter()
                    .map(|line| FileLineIndex::new(fid.clone(), line))
            })
            .collect()
    }
    pub fn new(file_lines: &Vec<FileLineIndex>) -> Self {
        let mut map: HashMap<FileIndex, Vec<LineIndex>> = HashMap::new();
        file_lines.iter().for_each(|fid_lid| {
            map.entry(fid_lid.file_id().clone())
                .or_insert(vec![])
                .push(fid_lid.line_id().clone());
        });
        Self {
            file_lines: map.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl FileLineData {
    pub fn get(&self) -> &String {
        &self.0
    }
    pub fn new(line: String) -> Self {
        Self(line)
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

impl IndexData {
    pub fn new() -> Self {
        Self {
            id_to_file: HashMap::new(),
            ngram_to_file_line: HashMap::new(),
        }
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
}

impl FromToData for NgramData {}
impl FromToData for FileLineData {}
impl FromToData for FileData {}

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
