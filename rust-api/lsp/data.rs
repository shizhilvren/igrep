use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
};

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

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub struct FileName {
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub struct DirName {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileData {
    file_content: FileContentData,
    semantic_tokens: Option<FileSemanticTokensData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileContentData {
    lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileSemanticTokensData {
    tokens: Vec<SemanticToken>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SemanticToken {
    pub delta_line: u32,
    pub delta_start: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers_bitset: u32,
}

impl FileSemanticTokensData {
    pub fn tokens(&self) -> &[SemanticToken] {
        &self.tokens
    }
}

impl FileContentData {
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl FileData {
    pub fn file_content(&self) -> &FileContentData {
        &self.file_content
    }
    pub fn semantic_tokens(&self) -> Option<&FileSemanticTokensData> {
        self.semantic_tokens.as_ref()
    }
}

impl DirData {
    pub fn files(&self) -> &[FileName] {
        &self.files
    }
    pub fn dirs(&self) -> &[DirName] {
        &self.dirs
    }
}

impl FileName {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl DirName {
    pub fn name(&self) -> &str {
        &self.name
    }
}



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

impl From<lsp_types::SemanticToken> for SemanticToken {
    fn from(value: lsp_types::SemanticToken) -> Self {
        Self {
            delta_line: value.delta_line,
            delta_start: value.delta_start,
            length: value.length,
            token_type: value.token_type,
            token_modifiers_bitset: value.token_modifiers_bitset,
        }
    }
}

impl From<lsp_types::SemanticTokens> for FileSemanticTokensData {
    fn from(value: lsp_types::SemanticTokens) -> Self {
        Self {
            tokens: value.data.into_iter().map(SemanticToken::from).collect(),
        }
    }
}

impl TryFrom<(FileContentData, Option<FileSemanticTokensData>)> for FileData {
    type Error = anyhow::Error;

    fn try_from(value: (FileContentData, Option<FileSemanticTokensData>)) -> Result<Self> {
        Ok(Self {
            file_content: value.0,
            semantic_tokens: value.1,
        })
    }
}

impl TryFrom<&FileIndex> for FileContentData {
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
