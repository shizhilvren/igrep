use crate::ngram::index::{FileIndex, FileLineIndex, FilesLinesIndex, LineIndex, NgramIndex};
use crate::ngram::path::{FileLinePath, FilePath, NgramPath};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GlobalData<'a> {
    ngram_len: u8,
    id_to_file: HashMap<FileIndex, FilePath>,
    ngram_to_file_line: HashMap<NgramIndex, NgramPath<'a>>,
}

pub struct FileData {
    file_path: String,
    file_name: String,
    lines_paths: HashMap<LineIndex, FileLinePath>,
}

#[derive(Serialize, Deserialize)]
pub struct NgramData {
    files_lines: FilesLinesIndex,
}

#[derive(Serialize, Deserialize)]
pub struct FileLineData {
    data: Vec<u8>,
}

impl From<FilesLinesIndex> for NgramData {
    fn from(value: FilesLinesIndex) -> Self {
        NgramData { files_lines: value }
    }
}

impl FromToData<'_> for NgramData {}

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
