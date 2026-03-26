use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};
use wasm_bindgen::prelude::*;

use crate::ngram::path::{FilePath, GetPath, NgramPath};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NgramIndex {
    ngaram: Box<[u8]>,
}

#[derive(Debug, Clone)]
pub struct NgramIndexVec(pub Vec<NgramIndex>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Deserialize)]

pub struct FileIndex {
    file_id: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LineIndex {
    line: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LinesIndex {
    lines_id: Vec<LineIndex>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct FileLinesIndex {
    file_id: FileIndex,
    lines_id: LinesIndex,
}

#[wasm_bindgen]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]

pub struct FilesLinesIndex {
    files_lines_id: Vec<FileLinesIndex>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileLineIndex {
    file_id: FileIndex,
    line_id: LineIndex,
}

impl NgramIndex {
    pub fn ngrams(&self) -> &[u8] {
        &self.ngaram
    }
}

impl FileIndex {
    pub fn file_id(&self) -> u32 {
        self.file_id
    }
}

impl LineIndex {
    pub fn line_id(&self) -> u32 {
        self.line
    }
    pub fn line_num(&self) -> u32 {
        self.line + 1
    }
}

impl LinesIndex {
    pub fn lines(&self) -> &[LineIndex] {
        &self.lines_id
    }
}
impl FileLinesIndex {
    pub fn file_id(&self) -> &FileIndex {
        &self.file_id
    }
    pub fn lines_index(&self) -> &LinesIndex {
        &self.lines_id
    }
}

impl FilesLinesIndex {
    pub fn files_lines(&self) -> &[FileLinesIndex] {
        &self.files_lines_id
    }
}

impl From<u32> for FileIndex {
    fn from(value: u32) -> Self {
        FileIndex { file_id: value }
    }
}

impl From<&[u8]> for NgramIndex {
    fn from(bytes: &[u8]) -> Self {
        NgramIndex {
            ngaram: bytes.into(),
        }
    }
}

impl From<Vec<NgramIndex>> for NgramIndexVec {
    fn from(value: Vec<NgramIndex>) -> Self {
        let mut sorted_value = value;
        sorted_value.sort();
        sorted_value.dedup();
        NgramIndexVec(sorted_value)
    }
}

impl From<(&[u8], u8)> for NgramIndexVec {
    fn from((bytes, ngram_len): (&[u8], u8)) -> Self {
        let ret = bytes
            .windows(ngram_len as usize)
            .map(|ngram| NgramIndex::from(ngram))
            .collect::<Vec<_>>();
        NgramIndexVec::from(ret)
    }
}

impl From<u32> for LineIndex {
    fn from(value: u32) -> Self {
        LineIndex { line: value }
    }
}

impl From<Vec<LineIndex>> for LinesIndex {
    fn from(value: Vec<LineIndex>) -> Self {
        let mut sorted_value = value;
        sorted_value.sort();
        sorted_value.dedup();
        LinesIndex {
            lines_id: sorted_value,
        }
    }
}

impl From<(FileIndex, LinesIndex)> for FileLinesIndex {
    fn from((file_id, lines_id): (FileIndex, LinesIndex)) -> Self {
        FileLinesIndex { file_id, lines_id }
    }
}

impl From<Vec<FileLinesIndex>> for FilesLinesIndex {
    fn from(value: Vec<FileLinesIndex>) -> Self {
        let mut sorted_value = value;
        sorted_value.sort_by_key(|file_lines_index| file_lines_index.file_id);
        sorted_value.dedup_by(|a, b| match a.file_id == b.file_id {
            true => {
                b.lines_id.lines_id.append(&mut a.lines_id.lines_id);
                b.lines_id.lines_id.sort();
                b.lines_id.lines_id.dedup();
                true
            }
            false => false,
        });
        FilesLinesIndex {
            files_lines_id: sorted_value,
        }
    }
}

impl SetCalculate for FilesLinesIndex {
    fn union(a: Self, b: Self) -> Self {
        let a = a.files_lines_id;
        let b = b.files_lines_id;
        let mut sorted_value = a;
        sorted_value.extend(b);
        FilesLinesIndex::from(sorted_value)
    }

    fn intersection(a: Self, b: Self) -> Self {
        let mut a_iter = a.files_lines_id.into_iter().peekable();
        let mut b_iter = b.files_lines_id.into_iter().peekable();
        let files_lines_id = std::iter::from_fn(|| {
            loop {
                match (a_iter.peek(), b_iter.peek()) {
                    (Some(a_file_lines), Some(b_file_lines)) => {
                        if a_file_lines.file_id == b_file_lines.file_id {
                            let a_file_lines = a_file_lines.clone();
                            let b_file_lines = b_file_lines.clone();
                            a_iter.next();
                            b_iter.next();
                            let lines = SetCalculate::intersection(
                                a_file_lines.lines_id,
                                b_file_lines.lines_id,
                            );
                            match lines.lines_id.is_empty() {
                                true => (),
                                false => {
                                    return Some(FileLinesIndex::from((
                                        a_file_lines.file_id,
                                        lines,
                                    )));
                                }
                            }
                        } else if a_file_lines.file_id < b_file_lines.file_id {
                            a_iter.next();
                        } else {
                            b_iter.next();
                        }
                    }
                    _ => return None,
                }
            }
        })
        .collect();
        FilesLinesIndex { files_lines_id }
    }
}

impl SetCalculate for LinesIndex {
    fn union(a: Self, b: Self) -> Self {
        let mut sorted_value = a.lines_id;
        sorted_value.extend(b.lines_id);
        LinesIndex::from(sorted_value)
    }

    fn intersection(a: Self, b: Self) -> Self {
        let mut a_iter = a.lines_id.into_iter().peekable();
        let mut b_iter = b.lines_id.into_iter().peekable();
        let lines_id = std::iter::from_fn(|| {
            loop {
                match (a_iter.peek(), b_iter.peek()) {
                    (Some(a_line), Some(b_line)) => {
                        if a_line == b_line {
                            let a_line = a_line.clone();
                            a_iter.next();
                            b_iter.next();
                            return Some(a_line);
                        } else if a_line < b_line {
                            a_iter.next();
                        } else {
                            b_iter.next();
                        }
                    }
                    _ => return None,
                }
            }
        })
        .collect();
        LinesIndex { lines_id }
    }
}

pub trait SetCalculate {
    fn union(a: Self, b: Self) -> Self;
    fn intersection(a: Self, b: Self) -> Self;
}

impl GetPath for NgramIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        NgramPath::from(self).path(base_path)
    }
}


impl GetPath for FileIndex {
    fn path(&self, base_path: &Path) -> PathBuf {
        FilePath::from(self).path(base_path)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn FilesLinesIndex_union() {
        let a = FilesLinesIndex::from(vec![
            FileLinesIndex::from((
                FileIndex::from(1),
                LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)]),
            )),
            FileLinesIndex::from((
                FileIndex::from(2),
                LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)]),
            )),
        ]);
        let b = FilesLinesIndex::from(vec![
            FileLinesIndex::from((
                FileIndex::from(1),
                LinesIndex::from(vec![LineIndex::from(2), LineIndex::from(3)]),
            )),
            FileLinesIndex::from((
                FileIndex::from(3),
                LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)]),
            )),
        ]);
        let c = FilesLinesIndex::union(a, b);
        assert_eq!(
            c.files_lines(),
            &vec![
                FileLinesIndex::from((
                    FileIndex::from(1),
                    LinesIndex::from(vec![
                        LineIndex::from(1),
                        LineIndex::from(2),
                        LineIndex::from(3)
                    ])
                )),
                FileLinesIndex::from((
                    FileIndex::from(2),
                    LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)])
                )),
                FileLinesIndex::from((
                    FileIndex::from(3),
                    LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)])
                )),
            ]
        );
    }

    #[test]
    fn FilesLinesIndex_from() {
        let a = FilesLinesIndex::from(vec![
            FileLinesIndex::from((
                FileIndex::from(1),
                LinesIndex::from(vec![LineIndex::from(1), LineIndex::from(2)]),
            )),
            FileLinesIndex::from((
                FileIndex::from(1),
                LinesIndex::from(vec![LineIndex::from(2), LineIndex::from(3)]),
            )),
        ]);
        assert_eq!(
            a.files_lines(),
            &vec![FileLinesIndex::from((
                FileIndex::from(1),
                LinesIndex::from(vec![
                    LineIndex::from(1),
                    LineIndex::from(2),
                    LineIndex::from(3)
                ])
            ))]
        );
    }
}
