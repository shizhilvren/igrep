use crate::index::{FileIndex, FileLineIndex, LineIndex, NgramIndex};
use crate::range::{FileLineRange, FileRange, NgramRange, Range};
use bincode::{self, Decode, Encode};
use flate2::Compression;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, ZlibEncoder};
use rayon::prelude::*;
use std::io::Write;
use std::io::prelude::*;
use std::path::Path;
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
    ngram_len: u8,
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
    pub fn new(ngram_len: u8) -> Self {
        Self {
            ngram_len,
            id_to_file: HashMap::new(),
            ngram_to_file_line: HashMap::new(),
        }
    }
    pub fn ngram_len(&self) -> u8 {
        self.ngram_len
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

impl FromToData for NgramData {}
impl FromToData for FileLineData {}
impl FromToData for FileData {}
impl FromToData for IndexData {}

pub trait FromToData {
    fn from_data(data: Vec<u8>) -> Result<Self, io::Error>
    where
        Self: Decode<()>,
    {
        let mut d = DeflateDecoder::new(data.as_slice());
        let mut buffer = Vec::new();
        match d.read_to_end(&mut buffer) {
            Ok(_) => bincode::decode_from_slice(&buffer, bincode::config::standard())
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to decode index data: {}", e),
                    )
                })
                .map(|(index_data, _)| index_data),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to decode index data: {}", e),
            )),
        }
    }
    fn to_data(&self) -> Result<Vec<u8>, io::Error>
    where
        Self: Encode,
    {
        bincode::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to encode index data: {}", e),
                )
            })
            .and_then(|data| {
                let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
                e.write_all(data.as_slice()).and_then(|_| e.finish())
            })
    }
}
