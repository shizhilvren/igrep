use std::path::{Path, PathBuf};




pub trait GetPath {
    fn path(&self, base_path: &Path) -> PathBuf;
    fn path_str(&self, base_path: &Path) -> String {
        self.path(base_path).to_string_lossy().into_owned()
    }
}