use bincode::{self, Decode, Encode};
use crate::data::FromToData;

#[derive(Debug, Decode, Encode, Clone, Copy)]
pub struct Range {
    pub start: Offset,
    pub len: u32,
}
pub type Offset = u64;

#[derive(Debug, Decode, Encode, Clone)]
pub struct FileLineRange(pub Range);

#[derive(Debug, Decode, Encode, Clone)]
pub struct FileRange(pub Range);

#[derive(Debug, Decode, Encode, Clone, Copy)]
pub struct NgramRange(pub Range);


impl Range {
    pub fn new(start: Offset, len: u32) -> Self {
        Range { start, len }
    }
}


impl FromToData for NgramRange {}
impl FromToData for FileRange {}
impl FromToData for FileLineRange {}