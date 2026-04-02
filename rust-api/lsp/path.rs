use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::lsp::{
    data::{FileData, TreeData},
    index::{FileIndex, PathIndex},
};
use anyhow::{Result, anyhow};
use log::debug;

use crate::lsp::data::FromToData;

pub struct TreeDataPath<'a> {
    full_path: &'a PathIndex,
}

impl TreeDataPath<'_> {
    pub fn dump(&self, base_path: &Path, tree_data: &TreeData) -> Result<()> {
        let path = self.path(base_path);
        debug!("Dump tree data to path: {:?}", &path);
        std::fs::create_dir_all(&path).map_err(|e| anyhow!("fail to create dir {:?} {:?}", &path, e))?;
        let file_path = match tree_data {
            TreeData::File(_) => path.join("file.data"),
            TreeData::Dir(_) => path.join("dir.data"),
        };
        let mut file = std::fs::File::create(file_path.as_path())
            .map_err(|e| anyhow!("create file {:?} fail. {:?}", self.full_path, e))?;
        let data = tree_data.to_data()?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl<'a> From<&'a PathIndex> for TreeDataPath<'a> {
    fn from(path_index: &'a PathIndex) -> Self {
        Self {
            full_path: path_index,
        }
    }
}

impl GetPath for TreeDataPath<'_> {
    fn path(&self, base_path: &Path) -> PathBuf {
        let index_path = self.full_path.path();
        debug!(
            "Get path for file index: {:?} with base path: {:?}",
            index_path, base_path
        );
        let index_path = match index_path.is_absolute() {
            true => index_path
                .strip_prefix("/")
                .expect("Failed to strip prefix"),
            false => index_path.as_path(),
        };
        base_path.join("index").join(index_path)
    }
}
pub trait GetPath {
    fn path(&self, base_path: &Path) -> PathBuf;
}
