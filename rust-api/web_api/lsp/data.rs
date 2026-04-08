use wasm_bindgen::prelude::*;

use crate::lsp::data::FromToData;

#[wasm_bindgen]
pub struct TreeData {
    data: crate::lsp::data::TreeData,
}

#[wasm_bindgen]
pub struct FileData {
    data: crate::lsp::data::FileData,
}

#[wasm_bindgen]
pub struct DirData {
    data: crate::lsp::data::DirData,
}

#[wasm_bindgen]
pub struct FileName {
    data: crate::lsp::data::FileName,
}

#[wasm_bindgen]
pub struct DirName {
    data: crate::lsp::data::DirName,
}

#[wasm_bindgen]
pub struct SemanticTokens {
    tokens: Vec<SemanticToken>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SemanticToken {
    data: crate::lsp::data::SemanticToken,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Position {
    line: u32,
    character: u32,
}

#[wasm_bindgen]
pub struct HoverRange {
    start: Position,
    end: Position,
}

#[wasm_bindgen]
impl Position {
    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn character(&self) -> u32 {
        self.character
    }
}

#[wasm_bindgen]
impl HoverRange {
    pub fn start(&self) -> Position {
        self.start.clone()
    }

    pub fn end(&self) -> Position {
        self.end.clone()
    }
}

#[wasm_bindgen]
pub struct HoversData {
    data: crate::lsp::data::HoversData,
}

#[wasm_bindgen]
pub struct HoverData {
    data: crate::lsp::data::HoverData,
}

#[wasm_bindgen]
impl HoversData {
    pub fn hovers(&self) -> Vec<HoverData> {
        self.data
            .hovers()
            .iter()
            .map(|h| HoverData::from(h.clone()))
            .collect()
    }
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Self {
        Self::try_from(&data).expect("data not correct")
    }
}

#[wasm_bindgen]
impl HoverData {
    pub fn range(&self) -> HoverRange {
        HoverRange::from(self.data.range())
    }

    pub fn hover(&self) -> String {
        self.data.hover().to_string()
    }
}

#[wasm_bindgen]
impl SemanticToken {
    pub fn delta_line(&self) -> u32 {
        self.data.delta_line
    }
    pub fn delta_start(&self) -> u32 {
        self.data.delta_start
    }
    pub fn length(&self) -> u32 {
        self.data.length
    }
    pub fn token_type(&self) -> u32 {
        self.data.token_type
    }
    pub fn token_modifiers_bitset(&self) -> u32 {
        self.data.token_modifiers_bitset
    }
}

#[wasm_bindgen]
impl TreeData {
    pub fn is_file(&self) -> bool {
        matches!(self.data, crate::lsp::data::TreeData::File(_))
    }
    pub fn is_dir(&self) -> bool {
        matches!(self.data, crate::lsp::data::TreeData::Dir(_))
    }

    pub fn file_data(self) -> Option<FileData> {
        match self.data {
            crate::lsp::data::TreeData::File(file_data) => Some(FileData::from(file_data)),
            _ => None,
        }
    }
    pub fn dir_data(self) -> Option<DirData> {
        match self.data {
            crate::lsp::data::TreeData::Dir(dir_data) => Some(DirData::from(dir_data)),
            _ => None,
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Self {
        Self::try_from(&data).expect("data not correct")
    }
}

#[wasm_bindgen]
impl FileData {
    pub fn lines(&self) -> Vec<String> {
        self.data.file_content().lines().iter().cloned().collect()
    }
    pub fn semantic_tokens(&self) -> Option<Vec<SemanticToken>> {
        self.data.semantic_tokens().map(|t| {
            t.tokens()
                .iter()
                .map(|t| SemanticToken::from(t.clone()))
                .collect()
        })
    }
}

#[wasm_bindgen]
impl DirData {
    pub fn files(&self) -> Vec<FileName> {
        self.data
            .files()
            .iter()
            .map(|f| FileName::from(f.clone()))
            .collect()
    }
    pub fn dirs(&self) -> Vec<DirName> {
        self.data
            .dirs()
            .iter()
            .map(|d| DirName::from(d.clone()))
            .collect()
    }
}

#[wasm_bindgen]
impl FileName {
    pub fn name(&self) -> String {
        self.data.name().to_string()
    }
}

#[wasm_bindgen]
impl DirName {
    pub fn name(&self) -> String {
        self.data.name().to_string()
    }
}

impl From<crate::lsp::data::TreeData> for TreeData {
    fn from(data: crate::lsp::data::TreeData) -> Self {
        Self { data }
    }
}
impl From<crate::lsp::data::FileData> for FileData {
    fn from(data: crate::lsp::data::FileData) -> Self {
        Self { data }
    }
}

impl From<crate::lsp::data::DirData> for DirData {
    fn from(data: crate::lsp::data::DirData) -> Self {
        Self { data }
    }
}

impl From<crate::lsp::data::FileName> for FileName {
    fn from(data: crate::lsp::data::FileName) -> Self {
        Self { data }
    }
}

impl From<crate::lsp::data::DirName> for DirName {
    fn from(data: crate::lsp::data::DirName) -> Self {
        Self { data }
    }
}
impl From<crate::lsp::data::SemanticToken> for SemanticToken {
    fn from(data: crate::lsp::data::SemanticToken) -> Self {
        Self { data }
    }
}

impl From<&lsp_types::Position> for Position {
    fn from(value: &lsp_types::Position) -> Self {
        Self {
            line: value.line,
            character: value.character,
        }
    }
}

impl From<&lsp_types::Range> for HoverRange {
    fn from(value: &lsp_types::Range) -> Self {
        Self {
            start: Position::from(&value.start),
            end: Position::from(&value.end),
        }
    }
}

impl TryFrom<&Vec<u8>> for HoversData {
    type Error = anyhow::Error;
    fn try_from(value: &Vec<u8>) -> anyhow::Result<Self> {
        let d = crate::lsp::data::HoversData::from_data(value.as_slice())?;
        Ok(Self { data: d })
    }
}

impl From<crate::lsp::data::HoverData> for HoverData {
    fn from(data: crate::lsp::data::HoverData) -> Self {
        Self { data }
    }
}

impl TryFrom<&Vec<u8>> for TreeData {
    type Error = anyhow::Error;
    fn try_from(value: &Vec<u8>) -> anyhow::Result<Self> {
        let d = crate::lsp::data::TreeData::from_data(value.as_slice())?;
        Ok(TreeData::from(d))
    }
}
