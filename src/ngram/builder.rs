use crate::ngram::data::NgramData;
use crate::ngram::path::FilePath;
use crate::ngram::{
    self,
    index::{
        FileIndex, FileLinesIndex, FilesLinesIndex, LineIndex, LinesIndex, NgramIndex,
        NgramIndexVec,
    },
    path::NgramPath,
};
use anyhow::{Error, Result, anyhow};
use bincode::de;
use log::{error, info, warn};
use rayon::prelude::*;
use std::path::Path;
use std::{collections::HashMap, default};

pub struct Builder {
    ngram_len: u8,
    ngram_to_files_lines: HashMap<NgramIndex, FilesLinesIndex>,
    file_id_to_content: HashMap<FileIndex, FileContent>,
}
pub struct FileIndexBuilder {
    file_to_id: HashMap<AbsPath, FileIndex>,
}

pub struct FileIndexFinalBuilder {
    files: Vec<(FileIndex, FileContent)>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct AbsPath {
    path: String,
}

pub struct FileContent {
    full_file_name: AbsPath,
    lines: Vec<String>,
}

pub struct BuilderOneIndex {
    file_id: FileIndex,
    file_content: FileContent,
    ngram_to_line: HashMap<NgramIndex, Vec<LineIndex>>,
}
impl Builder {
    pub fn new(ngram_len: u8) -> Result<Self, Error> {
        if ngram_len < 3 {
            return Err(anyhow!("Ngram length cannot be less than 3",));
        } else if ngram_len > 10 {
            return Err(anyhow!("Ngram length cannot be greater than 10",));
        } else {
            Ok(Self {
                ngram_len,
                ngram_to_files_lines: HashMap::new(),
                file_id_to_content: HashMap::new(),
            })
        }
    }

    pub fn index(&mut self, file_builder: FileIndexFinalBuilder) -> Result<()> {
        info!("start index files...");
        let all_builders = file_builder
            .files
            .into_par_iter()
            .map(|(file_id, file_content)| self.index_one(file_id, file_content))
            .collect::<Vec<_>>();
        info!("all file index finish. Start merging...");
        self.merge(all_builders)?;
        info!("all file merge finish.");
        Ok(())
    }

    pub fn dump(&self, base_path: &Path) -> Result<()> {
        self.dump_ngrams(base_path)?;
        self.dump_files(base_path)?;
        Ok(())
    }
}

impl Builder {
    fn index_one(&self, file_id: FileIndex, file_content: FileContent) -> BuilderOneIndex {
        let mut ngram_to_file_line: HashMap<NgramIndex, Vec<LineIndex>> = HashMap::new();
        file_content
            .lines
            .iter()
            .map(|line| NgramIndexVec::from((line.as_bytes(), self.ngram_len)))
            .enumerate()
            .map(|(id, ngrams)| (LineIndex::from((id) as u32), ngrams))
            .map(|(lid, ngrams)| ngrams.0.into_iter().map(move |ngram| (ngram, lid.clone())))
            .flatten()
            .for_each(|(ngram, lid)| {
                ngram_to_file_line
                    .entry(ngram)
                    .or_insert_with(Vec::new)
                    .push(lid);
            });
        BuilderOneIndex {
            file_id,
            file_content,
            ngram_to_line: ngram_to_file_line,
        }
    }
    fn merge(&mut self, file_index_to_content_ngram: Vec<BuilderOneIndex>) -> Result<()> {
        let mut ngram_to_files_lines = HashMap::new();
        file_index_to_content_ngram
            .into_iter()
            .map(
                |BuilderOneIndex {
                     file_id,
                     file_content,
                     ngram_to_line,
                 }| {
                    self.file_id_to_content
                        .insert(file_id, file_content)
                        .map_or(Ok(()), |old_content| {
                            Err(anyhow!(
                                "File with id {:?} is already indexed by path {:?}",
                                file_id,
                                old_content.full_file_name.path
                            ))
                        })
                        .and_then(move |()| {
                            Ok(ngram_to_line.into_iter().map(move |(ngram, line_ids)| {
                                (
                                    ngram,
                                    FileLinesIndex::from((file_id, LinesIndex::from(line_ids))),
                                )
                            }))
                        })
                },
            )
            .flatten()
            .flatten()
            .for_each(|(ngram, file_lines_index)| {
                ngram_to_files_lines
                    .entry(ngram)
                    .or_insert_with(Vec::new)
                    .push(file_lines_index);
            });
        self.ngram_to_files_lines = ngram_to_files_lines
            .into_iter()
            .map(|(k, v)| (k, FilesLinesIndex::from(v)))
            .collect::<HashMap<NgramIndex, FilesLinesIndex>>();
        Ok(())
    }
    fn dump_ngrams(
        &self,
        base_path: &Path,
    ) -> Result<()> {
        info!("start dump ngrams...");
        self.ngram_to_files_lines
            .par_iter()
            .try_for_each(|(ngram, files_lines)| {
                let ngarm_data = NgramData::from(files_lines.clone());
                let ngram_path = NgramPath::from(ngram);
                ngram_path.dump(base_path, &ngarm_data)
            })?;
        info!("dump ngrams finish.");
        Ok(())
    }
    fn dump_files(&self, base_path: &Path) -> Result<()> {
        info!("start dump files...");
        self.file_id_to_content
            .par_iter()
            .try_for_each(|(file_id, file_content)| {
                let file_path = FilePath::from(file_id);
                file_path.dump(base_path, file_content)
            })?;
        info!("dump files finish.");
        Ok(())
    }
}

impl FileIndexBuilder {
    pub fn new() -> Self {
        Self {
            file_to_id: HashMap::new(),
        }
    }
    pub fn build(&mut self, files_name_list: Vec<String>) -> Result<(), Error> {
        files_name_list
            .into_iter()
            .map(AbsPath::from)
            .try_for_each(|path| self.insert(&path))
    }
    fn insert(&mut self, path: &AbsPath) -> Result<(), Error> {
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

impl FileContent {
    pub fn get_full_file_name(&self) -> AbsPath {
        self.full_file_name.clone()
    }
    pub fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }
}

impl From<String> for AbsPath {
    fn from(path: String) -> Self {
        AbsPath { path }
    }
}

impl ToString for AbsPath {
    fn to_string(&self) -> String {
        self.path.clone()
    }
}

impl TryFrom<FileIndexBuilder> for FileIndexFinalBuilder {
    type Error = Error;
    fn try_from(builder: FileIndexBuilder) -> Result<Self, Self::Error> {
        info!("start reading files.");
        let files = builder
            .file_to_id
            .into_par_iter()
            .map(|(path, id)| (id, FileContent::try_from(path.clone())))
            .map(|(id, ret)| ret.map(|content| (id, content)))
            .filter_map(|result| match result {
                Ok(file) => Some(file),
                Err(e) => {
                    warn!("Failed to read file: {}", e);
                    None
                }
            })
            .collect::<Vec<(FileIndex, FileContent)>>();
        Ok(FileIndexFinalBuilder { files })
    }
}

impl TryFrom<AbsPath> for FileContent {
    type Error = Error;
    fn try_from(path: AbsPath) -> Result<Self, Self::Error> {
        let content = std::fs::read_to_string(&path.path)
            .map_err(|e| anyhow!("Failed to read file {}: {}", path.path, e))?;
        let lines = content.lines().map(String::from).collect();
        Ok(FileContent {
            full_file_name: path,
            lines,
        })
    }
}
