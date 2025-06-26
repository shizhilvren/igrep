use crate::index_builder::{FileIndex, FileLine,NgramIndex};
pub struct Range {
    pub start: usize,
    pub len: usize,
}

pub struct FileLineRange(Range);
pub struct AbsPathRange(Range);
pub struct AbsPathRange(Range);


pub struct Data {}

pub struct IndexData {
    size: usize,
    file_line: [(FileLine, FileLineRange)],
    id_to_file: [(FileIndex, AbsPathRange)],
    ngram_to_file_line: [(NgramIndex,)],
}
