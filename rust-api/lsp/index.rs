use std::path::{Path, PathBuf};

use crate::lsp::path::GetPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileIndex {
    full_path: PathBuf,
}

impl FileIndex {
    pub fn path(&self) -> PathBuf {
        self.full_path.clone()
    }
}

impl From<String> for FileIndex {
    fn from(value: String) -> Self {
        Self {
            full_path: PathBuf::from(value),
        }
    }
}
