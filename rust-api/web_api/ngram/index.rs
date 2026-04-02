use crate::ngram::path::GetPath as ngramGetPath;
use std::path::{Path, PathBuf};

use wasm_bindgen::prelude::*;

use crate::web_api::ngram::path::GetPath;

#[wasm_bindgen]
pub struct NgramIndexVec {
    vec: Vec<NgramIndex>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NgramIndex {
    ngram: crate::ngram::index::NgramIndex,
}

#[wasm_bindgen]
pub struct FileIndex {
    file_index: crate::ngram::index::FileIndex,
}

#[wasm_bindgen]
impl NgramIndexVec {
    pub fn vec(&self) -> Vec<NgramIndex> {
        self.vec.clone()
    }
}

#[wasm_bindgen]
impl NgramIndex {
    pub fn path_str(&self, base_path: &str) -> String {
        let base_path = Path::new(base_path);
        self.ngram.path(base_path).to_string_lossy().into_owned()
    }
}

#[wasm_bindgen]
impl FileIndex {
    pub fn path_str(&self, base_path: &str) -> String {
        let base_path = Path::new(base_path);
        self.file_index
            .path(base_path)
            .to_string_lossy()
            .into_owned()
    }
}

impl GetPath for NgramIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        self.ngram.path(base_path)
    }
}

impl GetPath for FileIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        self.file_index.path(base_path)
    }
}

impl Into<crate::ngram::index::NgramIndex> for NgramIndex {
    fn into(self) -> crate::ngram::index::NgramIndex {
        self.ngram
    }
}

impl Into<crate::ngram::index::FileIndex> for FileIndex {
    fn into(self) -> crate::ngram::index::FileIndex {
        self.file_index
    }
}

impl From<crate::ngram::index::NgramIndex> for NgramIndex {
    fn from(value: crate::ngram::index::NgramIndex) -> Self {
        NgramIndex { ngram: value }
    }
}

impl From<Vec<NgramIndex>> for NgramIndexVec {
    fn from(value: Vec<NgramIndex>) -> Self {
        NgramIndexVec { vec: value }
    }
}

impl From<crate::ngram::index::FileIndex> for FileIndex {
    fn from(value: crate::ngram::index::FileIndex) -> Self {
        FileIndex { file_index: value }
    }
}
