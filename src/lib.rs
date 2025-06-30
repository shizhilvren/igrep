pub mod config;
pub mod index_builder;
pub mod index_file;
pub mod index_regex;
use crate::{
    index_builder::NgramIndex,
    index_file::{FromToData, IndexData, NgramRange},
    index_regex::{Engine, NgramTree},
};
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_with_config() {
        let cfg = config::Config::new("path/to/config".to_string());
        assert_eq!(cfg.path(), "path/to/config");
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn build_index_data(data: Vec<u8>) -> IndexData {
    index_file::IndexData::from_data(data).unwrap()
}

#[wasm_bindgen]
pub fn index_regex_engine(pattern: &str) -> Option<Engine> {
    index_regex::Engine::new(pattern).map_or(None, |e| Some(e))
}

#[wasm_bindgen]
pub struct TreeNgramIndexRange {
    #[wasm_bindgen(skip)]
    pub tree: NgramTree,
    #[wasm_bindgen(skip)]
    index_ranges: Vec<NgramIndexRange>,
}

#[wasm_bindgen]
impl TreeNgramIndexRange {
    pub fn get_len(&self) -> usize {
        self.index_ranges.len()
    }
    pub fn range_at(&self, idx: usize) -> NgramRange {
        self.index_ranges.get(idx).unwrap().range
    }
}

#[wasm_bindgen]
pub struct NgramIndexRange {
    #[wasm_bindgen(skip)]
    index: NgramIndex,
    #[wasm_bindgen(skip)]
    pub range: NgramRange,
}

#[wasm_bindgen]
pub fn engine_build_tree(
    engine: &Engine,
    index_data: &IndexData,
    n: usize,
) -> Option<TreeNgramIndexRange> {
    let tree = engine.ngram(n);
    let all = tree.is_all();
    match all {
        true => None,
        false => {
            let index_ranges = tree
                .ngrams()
                .iter()
                .filter_map(|ngram_index| {
                    index_data.get_ngram_range(ngram_index).and_then(|range| {
                        Some(NgramIndexRange {
                            index: ngram_index.clone(),
                            range,
                        })
                    })
                })
                .collect::<Vec<_>>();
            Some(TreeNgramIndexRange { tree, index_ranges })
        }
    }
}
