
use crate::index::{FileIndex, FileLineIndex, LineIndex, NgramIndex};
use crate::range::{FileLineRange, FileRange, NgramRange, Range};
use crate::data::{FileData, FileLineData, NgramData};
use bincode::{self, Decode, Encode};
use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    io::{self, Error},
};
use wasm_bindgen::prelude::*;

pub struct Data {
    file_lines: Vec<(FileLineIndex, FileLineData)>,
    file_paths: HashMap<FileIndex, FileData>,
    ngrams: Vec<(NgramIndex, NgramData)>,
}

