use crate::data::FromToData;
use bincode::{self, Decode, Encode};
use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    io::{self, Error},
};
use wasm_bindgen::prelude::*;

/// This is NgramIndex, which is used to represent the index of n-grams in a file.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode, PartialOrd, Ord)]
pub struct NgramIndex {
    ngram: Box<[u8]>,
}

#[wasm_bindgen]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Decode, Encode, PartialOrd, Ord, Copy)]
pub struct FileIndex {
    file_id: u32,
}

#[wasm_bindgen]
#[derive(Clone, PartialEq, Debug, Decode, Encode, Hash, Eq, PartialOrd, Ord, Copy)]
pub struct LineIndex {
    line: u32,
}
#[wasm_bindgen]
#[derive(Clone, PartialEq, Debug, Decode, Encode, Hash, Eq, PartialOrd, Ord, Copy)]
pub struct FileLineIndex {
    #[wasm_bindgen(readonly)]
    pub file_id: FileIndex,
    #[wasm_bindgen(readonly)]
    pub line_id: LineIndex,
}

impl FileIndex {
    pub fn new(id: u32) -> Self {
        Self { file_id: id }
    }
}
impl NgramIndex {
    /// # Panics
    ///
    /// Panics if `n` size is zero.
    pub fn from_str(s: &[u8], n: u8) -> Vec<NgramIndex> {
        let mut ret = s
            .windows(n as usize)
            .map(|ngram| NgramIndex::new(ngram))
            .collect::<Vec<_>>();
        ret.sort();
        ret.dedup();
        ret
    }
    /// # Panics
    ///
    /// Panics if `ngram` size is zero.
    pub fn new(ngram: &[u8]) -> Self {
        match ngram.len() {
            0 => panic!("Ngram cannot be empty"),
            _ => NgramIndex {
                ngram: Box::from(ngram),
            },
        }
    }
}

#[wasm_bindgen]
impl LineIndex {
    pub fn new(line: u32) -> Self {
        if line == 0 {
            panic!("Line index cannot be zero");
        }
        Self { line }
    }
    pub fn line_number(&self) -> u32 {
        self.line
    }
}

impl FileLineIndex {
    pub fn new(file_id: FileIndex, line_id: LineIndex) -> Self {
        Self { file_id, line_id }
    }
    pub fn file_id(&self) -> &FileIndex {
        &self.file_id
    }
    pub fn line_id(&self) -> &LineIndex {
        &self.line_id
    }
}

impl FromToData for NgramIndex {}
impl FromToData for FileLineIndex {}
impl FromToData for FileIndex {}
