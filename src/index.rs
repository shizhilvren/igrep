use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, Error},
    path::PathBuf,
};


pub struct IndexBuilder {
    pub(crate) path: String,
}


struct FileContent {
    lines: Vec<String>,
}
struct FileLine {
    line_number: u32,
    content: String,
}


impl IndexBuilder {
    pub fn index(&mut self, file: String) -> Result<(), Error> {
        let path = PathBuf::from(file);
        let lines = fs::read_to_string(path)?
            .lines()
            .collect::<Vec<_>>();
        Ok(())
    }
}
