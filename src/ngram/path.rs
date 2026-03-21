use crate::ngram::{
    data::{FromToData, NgramData},
    index::NgramIndex,
};
use anyhow::{Result, anyhow};
use log::{error, info};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct FilePath {
    path: PathBuf,
}

pub struct NgramPath<'a> {
    ngram_index: &'a NgramIndex,
    ngram_data: &'a NgramData,
}

pub struct FileLinePath {
    path: PathBuf,
}

impl<'a> NgramPath<'a> {
    pub fn dump(&self, base_path: &Path) -> Result<()> {
        let path = self.path(base_path);
        let data = self.ngram_data.to_data()?;
        match path.parent() {
            Some(parent) => fs::create_dir_all(parent)
                .map_err(|e| anyhow!("crate ngram {:?} file fail. {:?}", self.ngram_index, e))?,
            None => {}
        };
        let mut file = fs::File::create(path.as_path())
            .map_err(|e| anyhow!("crate ngram {:?} file fail. {:?}", self.ngram_index, e))?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl<'a> GetPath for NgramPath<'a> {
    fn path(&self, base_path: &Path) -> PathBuf {
        let ngrams = self.ngram_index.get_ngrams();
        let ans = ngrams
            .into_iter()
            .map(|u| u.to_string())
            .collect::<PathBuf>();
        let path = base_path.join("ngrams").join(ans.as_path());
        path
    }
}

impl<'a> From<(&'a NgramIndex, &'a NgramData)> for NgramPath<'a> {
    fn from((ngram, ngram_data): (&'a NgramIndex, &'a NgramData)) -> Self {
        NgramPath {
            ngram_index: ngram,
            ngram_data: ngram_data,
        }
    }
}

pub trait GetPath {
    fn path(&self, base_path: &Path) -> PathBuf;
}
