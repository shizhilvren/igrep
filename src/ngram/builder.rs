use crate::ngram::index::{FileIndex, LineIndex, NgramIndex};
use anyhow::{Error, anyhow};
use bincode::de;
use std::{collections::HashMap, default};

pub struct FileIndexBuilder {
    file_to_id: HashMap<AbsPath, FileIndex>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct AbsPath {
    path: String,
}

impl FileIndexBuilder {
    pub fn new() -> Self {
        Self {
            file_to_id: HashMap::new(),
        }
    }
    pub fn insert(&mut self, path: &AbsPath) -> Result<(), Error> {
        let new_id = self.file_to_id.len() as u32;
        self.file_to_id
            .insert(path.clone(), FileIndex::from(new_id))
            .map_or_else(
                || Ok(()),
                |old_id| {
                    Err(anyhow!(
                        "File with path {} is already indexed by file id {:?}",
                        path.path,
                        old_id
                    ))
                },
            )
    }
}
