use anyhow::{Result, anyhow};
use log::{info, warn};
use rayon::iter::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path},
};

use crate::lsp::{
    data::{DirName, FileData, FileName, TreeData},
    index::{FileIndex, PathIndex},
    path::TreeDataPath,
};

pub struct Builder {
    // datas: Vec<FileBuilder>,
    datas: Vec<TreeBuilder>,
}

pub struct FileIndexBuilder {
    path_set: HashSet<FileIndex>,
}

pub struct FileBuilder {
    file_index: FileIndex,
    file_data: FileData,
}

pub struct TreeBuilder {
    path_index: PathIndex,
    tree_data: TreeData,
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
        self.dump_tree_data(base_path)?;
        Ok(())
    }
}

impl Builder {
    fn dump_tree_data(&self, base_path: &Path) -> Result<()> {
        self.datas
            .par_iter()
            .try_for_each(|tree_builder| tree_builder.dump(base_path))
    }
}

impl TreeBuilder {
    fn dump(&self, base_path: &Path) -> Result<()> {
        let tree_data_path = TreeDataPath::from(&self.path_index);
        tree_data_path.dump(base_path, &self.tree_data)
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

impl From<(PathIndex, TreeData)> for TreeBuilder {
    fn from(value: (PathIndex, TreeData)) -> Self {
        Self {
            path_index: value.0,
            tree_data: value.1,
        }
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

        let mut path_dir_set: HashMap<PathIndex, (HashSet<FileName>, HashSet<DirName>)> =
            HashMap::new();
        datas.iter().try_for_each(|a| {
            let file_index = a.file_index.clone();
            let ret = file_index
                .path()
                .ancestors()
                .skip(1)
                .filter_map(|p| {
                    let name = p.file_name().and_then(|n| n.to_str());
                    name.map(|name| (p.to_path_buf(), name.to_string()))
                })
                .try_for_each(|(p, name)| {
                    let is_dir = p.is_dir();
                    let is_file = p.is_file();
                    let p = p
                        .parent()
                        .map_or(
                            Err(anyhow!("{} mast have parent", &name)),
                            |_| Ok(p.clone()),
                        )
                        .and_then(|p| {
                            let path_index = PathIndex::from(p);
                            Ok((path_index, name, is_dir, is_file))
                        })
                        .and_then(|(path_index, name, is_dir, is_file)| {
                            let insert_ret = match (is_dir, is_file) {
                                (true, false) => {
                                    let ret = path_dir_set
                                        .entry(path_index)
                                        .or_insert((HashSet::new(), HashSet::new()))
                                        .1
                                        .insert(DirName::try_from(name)?);
                                    Ok(())
                                }
                                (false, true) => {
                                    let ret = path_dir_set
                                        .entry(path_index)
                                        .or_insert((HashSet::new(), HashSet::new()))
                                        .0
                                        .insert(FileName::try_from(name.clone())?);
                                    (!ret)
                                        .then(|| warn!("File {:?} already exists", name))
                                        .map_or_else(
                                            || Ok(()),
                                            |_| Err(anyhow!("File {:?} already exists", name)),
                                        )
                                }
                                _ => {
                                    warn!("Path {:?} is neither file nor directory", p);
                                    Ok(())
                                }
                            };
                            insert_ret
                        });
                    p
                });
            ret
        })?;

        let mut path_file_set: HashMap<PathIndex, FileData> = HashMap::new();
        datas.into_iter().try_for_each(|a| {
            let file_index = a.file_index;
            let file_data = a.file_data;
            let path = file_index.path();
            let path_index = PathIndex::from(path.clone());
            let ans = path_file_set
                .insert(path_index, file_data)
                .map_or(Ok(()), |_| Err(anyhow!("{:?} is exist", path)))?;
            Ok::<(), anyhow::Error>(())
        })?;
        let path_file_set = path_file_set
            .into_iter()
            .map(|(k, v)| (k, TreeData::File(v)))
            .collect::<HashMap<_, _>>();
        let path_dir_set = path_dir_set
            .into_iter()
            .map(|(k, v)| (k, TreeData::Dir(v.into())))
            .collect::<HashMap<_, _>>();
        path_file_set
            .keys()
            .any(|k| path_dir_set.contains_key(k))
            .then_some(())
            .map_or_else(|| Ok(()), |_| Err(anyhow!("some files in dirs.")))?;
        let path_set = path_file_set
            .into_iter()
            .chain(path_dir_set.into_iter())
            .map(TreeBuilder::from)
            .collect::<Vec<_>>();
        Ok(Self { datas: path_set })
    }
}
