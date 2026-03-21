use serde::{Deserialize, Serialize};
use std::{ops::Deref, str::FromStr};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NgramIndex {
    ngaram: Box<[u8]>,
}

pub struct NgramIndexVec(pub Vec<NgramIndex>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Deserialize)]

pub struct FileIndex {
    file_id: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LineIndex {
    line: u32,
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LinesIndex {
    lines_id: Vec<LineIndex>,
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct FileLinesIndex {
    file_id: FileIndex,
    lines_id: LinesIndex,
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]

pub struct FilesLinesIndex {
    files_lines_id: Vec<FileLinesIndex>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileLineIndex {
    file_id: FileIndex,
    line_id: LineIndex,
}

impl NgramIndex {
    pub fn get_ngrams(&self) -> &[u8] {
        &self.ngaram
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

impl From<(&[u8], u8)> for NgramIndexVec {
    fn from((bytes, ngram_len): (&[u8], u8)) -> Self {
        let mut ret = bytes
            .windows(ngram_len as usize)
            .map(|ngram| NgramIndex::from(ngram))
            .collect::<Vec<_>>();
        ret.sort();
        ret.dedup();
        NgramIndexVec(ret)
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
                a.lines_id.lines_id.append(&mut b.lines_id.lines_id);
                a.lines_id.lines_id.sort();
                a.lines_id.lines_id.dedup();
                true
            }
            false => false,
        });
        FilesLinesIndex {
            files_lines_id: sorted_value,
        }
    }
}
