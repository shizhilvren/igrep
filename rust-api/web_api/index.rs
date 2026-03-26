use crate::ngram::path::GetPath as ngramGetPath;
use std::path::{Path, PathBuf};

use wasm_bindgen::prelude::*;

use crate::web_api::path::GetPath;

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

impl GetPath for NgramIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        self.ngram.path(base_path)
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
