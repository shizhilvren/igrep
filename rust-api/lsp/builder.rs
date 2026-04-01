use anyhow::{Result, anyhow};
use log::{info, warn};
use rayon::iter::*;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::lsp::{data::FileData, index::FileIndex, path::FileDataPath};

pub struct Builder {
    datas: Vec<FileBuilder>,
}

pub struct FileIndexBuilder {
    path_set: HashSet<FileIndex>,
}

pub struct FileBuilder {
    file_index: FileIndex,
    file_data: FileData,
}

impl FileIndexBuilder {
    pub fn insert(&mut self, path: FileIndex) -> Result<()> {
        match self.path_set.insert(path.clone()) {
            true => Ok(()),
            false => {
                warn!("Path {:?} already exists", &path);
                Err(anyhow!("Path {:?} already exists", &path))
            }
        }
    }
}

impl Builder {
    pub fn dump(&self, base_path: &Path) -> Result<()> {
        info!("Dumping LSP index to {:?}", base_path);
        if base_path.exists() {
            info!("Removing old dump directory: {:?}", base_path);
            std::fs::remove_dir_all(base_path).map_err(|e| {
                anyhow!("Failed to remove old dump directory {:?}: {}", base_path, e)
            })?;
        }
        self.dump_file_data(base_path)?;
        Ok(())
    }
}

impl Builder {
    fn dump_file_data(&self, base_path: &Path) -> Result<()> {
        self.datas
            .par_iter()
            .try_for_each(|file_builder| file_builder.dump(base_path))
    }
}

impl FileBuilder {
    fn dump(&self, base_path: &Path) -> Result<()> {
        let file_data_path = FileDataPath::from(&self.file_index);
        file_data_path.dump(base_path, &self.file_data)
    }
}

impl From<()> for FileIndexBuilder {
    fn from(_: ()) -> Self {
        Self {
            path_set: HashSet::new(),
        }
    }
}

impl TryFrom<FileIndex> for FileBuilder {
    type Error = anyhow::Error;

    fn try_from(value: FileIndex) -> Result<Self> {
        let data = FileData::try_from(&value)?;
        Ok(Self {
            file_data: data,
            file_index: value,
        })
    }
}

impl TryFrom<FileIndexBuilder> for Builder {
    type Error = anyhow::Error;

    fn try_from(value: FileIndexBuilder) -> Result<Self> {
        let datas = value
            .path_set
            .into_par_iter()
            .filter_map(|index| {
                FileBuilder::try_from(index.clone()).map_or_else(
                    |e| {
                        warn!(
                            "Failed to create FileBuilder for index {:?}, error: {:?}",
                            index, e
                        );
                        None
                    },
                    |d| Some(d),
                )
            })
            .collect::<Vec<FileBuilder>>();
        Ok(Self { datas })
    }
}
