use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::lsp::{
    data::{DefinitionsData, FileData, HoversData, TreeData},
    index::{FileIndex, PathIndex},
};
use anyhow::{Result, anyhow};
use log::debug;

use crate::lsp::data::FromToData;

pub struct TreeDataPath<'a> {
    full_path: &'a PathIndex,
}

pub struct HoverDataPath<'a> {
    full_path: &'a FileIndex,
}

pub struct DefinitionDataPath<'a> {
    full_path: &'a FileIndex,
}

impl TreeDataPath<'_> {
    pub fn dump(&self, base_path: &Path, tree_data: &TreeData) -> Result<()> {
        let path = self.path(base_path);
        // debug!("Dump tree data to path: {:?}", &path);
        std::fs::create_dir_all(&path)
            .map_err(|e| anyhow!("fail to create dir {:?} {:?}", &path, e))?;
        let file_path = match tree_data {
            TreeData::File(_) => path.join("tree.data"),
            TreeData::Dir(_) => path.join("tree.data"),
        };
        let mut file = std::fs::File::create(file_path.as_path())
            .map_err(|e| anyhow!("create file {:?} fail. {:?}", self.full_path, e))?;
        let data = tree_data.to_data()?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl HoverDataPath<'_> {
    pub fn dump(&self, base_path: &Path, hovers_data: &HoversData) -> Result<()> {
        let path = self.path(base_path);
        // debug!("Dump hover data to path: {:?}", &path);
        std::fs::create_dir_all(&path)
            .map_err(|e| anyhow!("fail to create dir {:?} {:?}", &path, e))?;
        let file_path = path.join("hover.data");
        let mut file = std::fs::File::create(file_path.as_path())
            .map_err(|e| anyhow!("create file {:?} fail. {:?}", self.full_path, e))?;
        let data = hovers_data.to_data()?;
        file.write_all(&data)?;
        Ok(())
    }
}
impl DefinitionDataPath<'_> {
    pub fn dump(&self, base_path: &Path, definitions_data: &DefinitionsData) -> Result<()> {
        let path = self.path(base_path);
        // debug!("Dump definition data to path: {:?}", &path);
        std::fs::create_dir_all(&path)
            .map_err(|e| anyhow!("fail to create dir {:?} {:?}", &path, e))?;
        let file_path = path.join("definition.data");
        let mut file = std::fs::File::create(file_path.as_path())
            .map_err(|e| anyhow!("create file {:?} fail. {:?}", self.full_path, e))?;
        let data = definitions_data.to_data()?;
        file.write_all(&data)?;
        Ok(())
    }
}



impl<'a> From<&'a FileIndex> for DefinitionDataPath<'a> {
    fn from(file_index: &'a FileIndex) -> Self {
        Self {
            full_path: file_index,
        }
    }
}

impl<'a> From<&'a FileIndex> for HoverDataPath<'a> {
    fn from(file_index: &'a FileIndex) -> Self {
        Self {
            full_path: file_index,
        }
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
        let index_path = match index_path.is_absolute() {
            true => index_path
                .strip_prefix("/")
                .expect("Failed to strip prefix"),
            false => index_path.as_path(),
        };
        base_path.join("index").join(index_path)
    }
}

impl GetPath for HoverDataPath<'_> {
    fn path(&self, base_path: &Path) -> PathBuf {
        let index_path = self.full_path.path();
        let index_path = match index_path.is_absolute() {
            true => index_path
                .strip_prefix("/")
                .expect("Failed to strip prefix"),
            false => index_path.as_path(),
        };
        base_path.join("index").join(index_path)
    }
}

impl GetPath for DefinitionDataPath<'_> {
    fn path(&self, base_path: &Path) -> PathBuf {
        let index_path = self.full_path.path();
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
