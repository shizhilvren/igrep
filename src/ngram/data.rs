use crate::ngram::builder::FileContent;
use crate::ngram::index::{FileIndex, FileLineIndex, FilesLinesIndex, LineIndex, NgramIndex};
use crate::ngram::path::{FileLinePath, FilePath, NgramPath};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GlobalData<'a> {
    ngram_len: u8,
    id_to_file: HashMap<FileIndex, FilePath<'a>>,
    ngram_to_file_line: HashMap<NgramIndex, NgramPath<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct FileData {
    // file_path: String,
    // file_name: String,
    full_file_name: String,
    lines_paths: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NgramData {
    files_lines: FilesLinesIndex,
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
