pub struct NgramIndex{
    ngaram: Box<[u8]>,
}

pub struct FileIndex {
    file_id: u32,
}

pub struct LineIndex {
    line: u32,
}

pub struct LinesIndex {
    lines_id: Vec<LineIndex>,
}

pub struct FileLinesIndex {
    file_id: FileIndex,
    lines_id: LinesIndex,
}

pub struct FilesLinesIndex {
    files_lines_id: Vec<FileLinesIndex>,
}

pub struct FileLineIndex {
    file_id: FileIndex,
    line_id: LineIndex,
}
