use crate::ngram::{
    builder::FileContent,
    data::{FileData, FromToData, NgramData},
    index::{FileIndex, NgramIndex},
};
use anyhow::{Result, anyhow};
use log::{error, info};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct FilePath<'a> {
    file_index: &'a FileIndex,
}

pub struct NgramPath<'a> {
    ngram_index: &'a NgramIndex,
}

pub struct FileLinePath {
    path: PathBuf,
}

impl<'a> NgramPath<'a> {
    pub fn dump(&self, base_path: &Path, ngram_data: &NgramData) -> Result<()> {
        let path = self.path(base_path);
        let data = ngram_data.to_data()?;
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

impl<'a> FilePath<'a> {
    pub fn dump(&self, base_path: &Path, file_content: &FileContent) -> Result<()> {
        let path_dir = self.path(base_path);
        fs::create_dir_all(&path_dir)
            .map_err(|e| anyhow!("crate file {:?} fail. {:?}", self.file_index, e))?;
        let file = path_dir.join("file");
        let mut file = fs::File::create(file.as_path())
            .map_err(|e| anyhow!("crate file {:?} fail. {:?}", self.file_index, e))?;
        let data = FileData::from(file_content).to_data()?;
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

impl<'a> GetPath for FilePath<'a> {
    fn path(&self, base_path: &Path) -> PathBuf {
        let id = self.file_index.get_file_id();
        let hash_id = id % 1024;
        let path = base_path
            .join("files")
            .join(hash_id.to_string())
            .join(id.to_string());
        path
    }
}

impl<'a> From<&'a NgramIndex> for NgramPath<'a> {
    fn from(ngram: &'a NgramIndex) -> Self {
        NgramPath {
            ngram_index: ngram,
        }
    }
}

impl<'a> From<&'a FileIndex> for FilePath<'a> {
    fn from(file_index: &'a FileIndex) -> Self {
        FilePath { file_index }
    }
}

pub trait GetPath {
    fn path(&self, base_path: &Path) -> PathBuf;
}
