use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_namespace = "lsp")]
pub struct TreeData {
    data: crate::lsp::data::TreeData,
}

#[wasm_bindgen(js_namespace = "lsp")]
pub struct FileData {
    data: crate::lsp::data::FileData,
}

#[wasm_bindgen(js_namespace = "lsp")]
pub struct DirData {
    data: crate::lsp::data::DirData,
}

#[wasm_bindgen(js_namespace = "lsp")]
pub struct FileName {
    data: crate::lsp::data::FileName,
}

#[wasm_bindgen(js_namespace = "lsp")]
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
}

#[wasm_bindgen]
impl FileData {
    pub fn lines(&self) -> Vec<String> {
        self.data.lines().iter().cloned().collect()
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
