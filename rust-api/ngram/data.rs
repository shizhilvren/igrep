use crate::ngram::builder::FileContent;
use crate::ngram::index::{FileIndex, FileLineIndex, FilesLinesIndex, LineIndex, NgramIndex};
use crate::ngram::path::{FilePath, NgramPath};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone) ]
pub struct GlobalData {
    ngram_len: u8,
    indexs: HashSet<NgramIndex>,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct FileData {
    // file_path: String,
    // file_name: String,
    full_file_name: String,
    lines_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NgramData {
    files_lines: FilesLinesIndex,
}

impl FileData {
    pub fn full_file_name(&self) -> &str {
        &self.full_file_name
    }
    pub fn lines(&self, line_index: &LineIndex) -> Option<&String> {
        self.lines_paths.get(line_index.line_id() as usize)
    }
}

impl GlobalData {
    pub fn ngram_len(&self) -> u8 {
        self.ngram_len
    }
    pub fn has_index(&self, index: &NgramIndex) -> bool {
        self.indexs.contains(index)
    }
}

impl NgramData {
    pub fn files_lines(&self) -> &FilesLinesIndex {
        &self.files_lines
    }
}

impl From<FilesLinesIndex> for NgramData {
    fn from(value: FilesLinesIndex) -> Self {
        NgramData { files_lines: value }
    }
}

impl From<&FileContent> for FileData {
    fn from(value: &FileContent) -> Self {
        FileData {
            full_file_name: value.get_full_file_name().to_string(),
            lines_paths: value.get_lines().clone(),
        }
    }
}

impl From<(u8, HashSet<NgramIndex>)> for GlobalData {
    fn from((value, indexs): (u8, HashSet<NgramIndex>)) -> Self {
        GlobalData {
            ngram_len: value,
            indexs: indexs,
        }
    }
}

impl FromToData<'_> for GlobalData {}
impl FromToData<'_> for NgramData {}
impl FromToData<'_> for FileData {}

pub trait FromToData<'a> {
    fn to_data(&self) -> Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let ret = postcard::to_stdvec(&self)?;
        Ok(ret)
    }
    fn from_data(data: &'a [u8]) -> Result<Self>
    where
        Self: Deserialize<'a>,
    {
        let ans = postcard::from_bytes(data)?;
        Ok(ans)
    }
}
