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

impl TryFrom<&Vec<u8>> for TreeData {
    type Error = anyhow::Error;
    fn try_from(value: &Vec<u8>) -> anyhow::Result<Self> {
        let d = crate::lsp::data::TreeData::from_data(value.as_slice())?;
        Ok(TreeData::from(d))
    }
}
