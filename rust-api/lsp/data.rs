use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs};

use crate::lsp::index::FileIndex;

#[derive(Serialize, Deserialize, Debug)]
pub enum TreeData {
    File(FileData),
    Dir(DirData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirData {
    files: Vec<FileName>,
    dirs: Vec<DirName>,
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct FileName {
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct DirName {
    name: String,
}

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
impl FromToData<'_> for TreeData {}
impl FromToData<'_> for DirData {}

impl TryFrom<String> for FileName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        let path: &std::path::Path = std::path::Path::new(&value);
        let level = value.find("/");
        level.map_or_else(
            || Ok(()),
            |a| {
                Err(anyhow!(
                    "Invalid file name: {:?} {a}, it should contain at least one directory level",
                    value
                ))
            },
        )?;
        path.has_root().then(|| ()).map_or_else(
            || Ok(()),
            |_| {
                Err(anyhow!(
                    "Invalid file na already existsme have root: {:?} ",
                    value
                ))
            },
        )?;
        Ok(Self { name: value })
    }
}

impl TryFrom<String> for DirName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        let path: &std::path::Path = std::path::Path::new(&value);
        let level = value.find("/");
        level.map_or_else(
            || Ok(()),

            |a| {
                Err(anyhow!(
                    "Invalid directory name: {:?} {a}, it should contain at least one directory level",
                    value
                ))
            },
        )?;
        path.has_root().then(|| ()).map_or_else(
            || Ok(()),
            |_| Err(anyhow!("Invalid directory name: {:?}", value)),
        )?;
        Ok(Self { name: value })
    }
}

impl From<(HashSet<FileName>, HashSet<DirName>)> for DirData {
    fn from(value: (HashSet<FileName>, HashSet<DirName>)) -> Self {
        Self {
            files: value.0.into_iter().collect(),
            dirs: value.1.into_iter().collect(),
        }
    }
}

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
