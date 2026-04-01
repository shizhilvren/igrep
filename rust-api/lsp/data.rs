use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::lsp::index::FileIndex;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileData {
    lines: Vec<String>,
}

impl TryFrom<&FileIndex> for FileData {
    type Error = anyhow::Error;

    fn try_from(file_index: &FileIndex) -> Result<Self> {
        let lines = fs::read_to_string(file_index.path())
            .map_err(|e| {
                anyhow!(
                    "Failed to read file at path: {:?}, error: {:?}",
                    file_index.path(),
                    e
                )
            })?
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(Self { lines })
    }
}

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
