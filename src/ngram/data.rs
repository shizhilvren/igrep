use crate::ngram::index::{FileIndex, FileLineIndex, FilesLinesIndex, LineIndex, NgramIndex};
use crate::ngram::path::{FileLinePath, FilePath, NgramPath};
use std::collections::HashMap;

pub struct GlobalData<'a> {
    ngram_len: u8,
    id_to_file: HashMap<FileIndex, FilePath>,
    ngram_to_file_line: HashMap<NgramIndex, NgramPath<'a>>,
}

pub struct FileData {
    file_path: String,
    file_name: String,
    lines_paths: HashMap<LineIndex, FileLinePath>,
}

pub struct NgramData {
    files_lines: FilesLinesIndex,
}

pub struct FileLineData {
    data: Vec<u8>,
}
