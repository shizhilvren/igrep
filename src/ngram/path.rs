use std::path::{Path, PathBuf};
use crate::ngram::index::NgramIndex;

pub struct FilePath {
    path: PathBuf,
}

pub struct NgramPath<'a> {
    ngram_index: &'a NgramIndex,
}

pub struct FileLinePath {
    path: PathBuf,
}

