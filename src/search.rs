use crate::{
    index::{FileLineIndex, NgramIndex},
};
use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    result,
};
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub struct Engine {
    regex: Hir,
}

impl Engine {
    pub fn new(pattern: &str) -> Result<Self, regex_syntax::Error> {
        let regex = parse(pattern)?;
        Ok(Self { regex })
    }
}
