use bincode::{self, Decode, Encode};
use wasm_bindgen::prelude::*;
use crate::data::FromToData;

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone, Copy)]
pub struct Range {
    #[wasm_bindgen(readonly)]
    pub start: Offset,
    #[wasm_bindgen(readonly)]
    pub len: u32,
}
pub type Offset = u64;

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone)]
pub struct FileLineRange(pub Range);

#[wasm_bindgen]
#[derive(Debug, Decode, Encode, Clone)]
pub struct FileRange(pub Range);

#[wasm_bindgen]
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