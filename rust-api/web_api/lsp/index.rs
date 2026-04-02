use std::path::{Path, PathBuf};

use wasm_bindgen::prelude::*;

use crate::lsp::path::GetPath as lspGetPath;
use crate::{lsp::path::TreeDataPath, web_api::lsp::path::GetPath};

#[wasm_bindgen]
pub struct PathIndex {
    index: crate::lsp::index::PathIndex,
}

#[wasm_bindgen]
impl PathIndex {
    pub fn path_str(&self, base_path: &str) -> String {
        let base_path = Path::new(base_path);
        self.path(base_path).to_string_lossy().into_owned()
    }
    #[wasm_bindgen(constructor)]
    pub fn new(path: String) -> Self {
        Self::from(path)
    }
}



impl From<String> for PathIndex {
    fn from(path: String) -> Self {
        Self {
            index: crate::lsp::index::PathIndex::from(PathBuf::from(path)),
        }
    }
}


impl GetPath for PathIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        TreeDataPath::from(&self.index).path(base_path)
    }
}
