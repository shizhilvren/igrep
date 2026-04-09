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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HoverData {
    range: lsp_types::Range,
    hover: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HoversData {
    hovers: Vec<HoverData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefinitionsData {
    definitions: Vec<DefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefinitionData {
    range: lsp_types::Range,
    locations: Vec<LocationData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationData {
    range: lsp_types::Range,
    file_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SemanticToken {
    pub delta_line: u32,
    pub delta_start: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers_bitset: u32,
}

impl LocationData {
    pub fn range(&self) -> &lsp_types::Range {
        &self.range
    }
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl DefinitionsData {
    pub fn definitions(&self) -> &[DefinitionData] {
        &self.definitions
    }
}
impl DefinitionData {
    pub fn range(&self) -> &lsp_types::Range {
        &self.range
    }
    pub fn locations(&self) -> &[LocationData] {
        &self.locations
    }
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

impl HoversData {
    pub fn hovers(&self) -> &[HoverData] {
        &self.hovers
    }
    pub fn hover(&self) -> Option<&HoverData> {
        self.hovers.first()
    }
}

impl HoverData {
    pub fn hover(&self) -> &str {
        &self.hover
    }
    pub fn range(&self) -> &lsp_types::Range {
        &self.range
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

impl TryFrom<lsp_types::Hover> for HoverData {
    type Error = anyhow::Error;

    fn try_from(value: lsp_types::Hover) -> Result<Self> {
        match (value.contents, value.range) {
            (lsp_types::HoverContents::Markup(markup), Some(range)) => Ok(Self {
                range,
                hover: markup.value,
            }),
            _ => Err(anyhow!("Hover data is missing range")),
        }
    }
}

impl TryFrom<Vec<lsp_types::Hover>> for HoversData {
    type Error = anyhow::Error;

    fn try_from(value: Vec<lsp_types::Hover>) -> Result<Self> {
        let mut hovers = value
            .into_iter()
            .map(HoverData::try_from)
            .collect::<Result<Vec<_>>>()?;
        hovers.sort_by(|a, b| {
            a.range
                .start
                .line
                .cmp(&b.range.start.line)
                .then(a.range.start.character.cmp(&b.range.start.character))
        });
        // let ans = hovers.windows(2).try_for_each(|[a, b]| {
        //     assert!(
        //         a.range.start.line < b.range.start.line
        //             || (a.range.start.line == b.range.start.line
        //                 && a.range.start.character < b.range.start.character)
        //     );
        //     Ok(())
        // })?;
        Ok(Self { hovers })
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

impl TryFrom<lsp_types::Location> for LocationData {
    type Error = anyhow::Error;

    fn try_from(value: lsp_types::Location) -> std::result::Result<Self, Self::Error> {
        let file_name = value
            .uri
            .as_str()
            .strip_prefix("file://")
            .ok_or(anyhow!(
                "Invalid file URI: {:?}, expected to start with 'file://'",
                value.uri
            ))?
            .to_string();
        Ok(Self {
            range: value.range,
            file_name: file_name,
        })
    }
}

impl TryFrom<(lsp_types::Range, Vec<lsp_types::Location>)> for DefinitionData {
    type Error = anyhow::Error;
    fn try_from(
        (range, value): (lsp_types::Range, Vec<lsp_types::Location>),
    ) -> std::result::Result<Self, Self::Error> {
        value
            .into_iter()
            .map(LocationData::try_from)
            .collect::<Result<Vec<_>>>()
            .map(|locations| Self { range, locations })
    }
}

impl From<Vec<DefinitionData>> for DefinitionsData {
    fn from(value: Vec<DefinitionData>) -> Self {
        Self { definitions: value }
    }
}

impl FromToData<'_> for FileData {}
impl FromToData<'_> for TreeData {}
impl FromToData<'_> for DirData {}
impl FromToData<'_> for HoversData {}
impl FromToData<'_> for DefinitionsData {}

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
