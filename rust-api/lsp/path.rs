use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::lsp::{data::FileData, index::FileIndex};
use anyhow::{Result, anyhow};
use log::debug;

use crate::lsp::data::FromToData;
pub struct FileDataPath<'a> {
    full_path: &'a FileIndex,
}

impl FileDataPath<'_> {
    pub fn dump(&self, base_path: &Path, file_data: &FileData) -> Result<()> {
        let path = self.path(base_path, self.full_path);
        debug!("Dump file data to path: {:?}", &path);
        match path.parent() {
            Some(parent) => std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("create directory {:?} fail. {:?}", parent, e))?,
            None => {}
        };
        let mut file = std::fs::File::create(path.as_path())
            .map_err(|e| anyhow!("create file {:?} fail. {:?}", self.full_path, e))?;
        let data = file_data.to_data()?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl<'a> From<&'a FileIndex> for FileDataPath<'a> {
    fn from(file_index: &'a FileIndex) -> Self {
        Self {
            full_path: file_index,
        }
    }
}

impl GetPath for FileDataPath<'_> {
    fn path(&self, base_path: &Path, index: &FileIndex) -> PathBuf {
        debug!(
            "Get path for file index: {:?} with base path: {:?}",
            index.path(),
            base_path
        );
        let index_path = index.path();
        let index_path = match index_path.is_absolute() {
            true => index_path
                .strip_prefix("/")
                .expect("Failed to strip prefix"),
            false => index_path.as_path(),
        };
        base_path.join("file").join(index_path)
    }
}
pub trait GetPath {
    fn path(&self, base_path: &Path, index: &FileIndex) -> PathBuf;
}
