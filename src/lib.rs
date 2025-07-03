pub mod builder;
pub mod config;
pub mod data;
pub mod encode;
pub mod index;
pub mod index_file;
pub mod index_regex;
pub mod range;
pub mod search;

use wasm_bindgen::prelude::*;

use crate::range::{NgramRange, Range};

#[wasm_bindgen]
pub struct A {
    pub a: u8,
    pub b: NgramRange,
}

#[wasm_bindgen]
pub fn fun() -> Vec<A> {
    vec![A {
        a: 1,
        b: NgramRange(Range::new(1, 10)),
    }]
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works_with_config() {
//         let cfg = config::Config::new("path/to/config".to_string());
//         assert_eq!(cfg.path(), "path/to/config");
//     }
// }

// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);

//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// #[wasm_bindgen]
// pub fn build_index_data(data: Vec<u8>) -> IndexData {
//     let data = index_file::IndexData::from_data(data);
//     log(&format!("build index data with {:?} file lines", &data));
//     data.unwrap()
// }

// #[wasm_bindgen]
// pub fn index_regex_engine(pattern: &str) -> Option<Engine> {
//     index_regex::Engine::new(pattern).map_or(None, |e| Some(e))
// }

// #[wasm_bindgen]
// pub struct TreeNgramIndexRange {
//     #[wasm_bindgen(skip)]
//     pub tree: NgramTree,
//     #[wasm_bindgen(skip)]
//     index_ranges: Vec<NgramIndexRange>,
// }

// #[wasm_bindgen]
// pub struct NgramTreeResult {
//     ans: index_regex::NgramTreeResult,
// }

// #[wasm_bindgen]
// impl NgramTreeResult {
//     pub fn len(&self) -> usize {
//         match &self.ans {
//             index_regex::NgramTreeResult::ALL => panic!(),
//             index_regex::NgramTreeResult::Set(set) => set.len(),
//         }
//     }
//     pub fn get(&self, idx: usize) -> FileLineIndex {
//         match &self.ans {
//             index_regex::NgramTreeResult::ALL => panic!(),
//             index_regex::NgramTreeResult::Set(set) => set.iter().nth(idx).unwrap().clone(),
//         }
//     }
//     pub fn all(&self) -> bool {
//         match self.ans {
//             index_regex::NgramTreeResult::ALL => true,
//             _ => false,
//         }
//     }
// }

// #[wasm_bindgen]
// impl TreeNgramIndexRange {
//     pub fn get_len(&self) -> usize {
//         self.index_ranges.len()
//     }
//     pub fn range_at(&self, idx: usize) -> NgramRange {
//         self.index_ranges.get(idx).unwrap().range
//     }
//     pub fn set_data_at(&mut self, idx: usize, data: Vec<u8>) {
//         let data = NgramData::from_data(data).unwrap();
//         log(&format!(
//             "set data at {} {:?} range {:?} {} file lines",
//             idx,
//             self.index_ranges.get(idx).unwrap().index,
//             self.index_ranges.get(idx).unwrap().range,
//             data.file_lines().len()
//         ));
//         self.index_ranges.get_mut(idx).unwrap().data = data;
//     }

//     pub fn search(&self) -> NgramTreeResult {
//         let map = self
//             .index_ranges
//             .iter()
//             .map(|ird| (ird.index.clone(), &ird.data))
//             .collect::<HashMap<_, _>>();
//         NgramTreeResult {
//             ans: self.tree.get_file_lines(&map),
//         }
//     }
// }

// #[wasm_bindgen]
// pub struct NgramIndexRange {
//     #[wasm_bindgen(skip)]
//     index: NgramIndex,
//     #[wasm_bindgen(skip)]
//     pub range: NgramRange,
//     #[wasm_bindgen(skip)]
//     pub data: NgramData,
// }

// #[wasm_bindgen]
// pub fn engine_build_tree(
//     engine: &Engine,
//     index_data: &IndexData,
//     n: u8,
// ) -> Option<TreeNgramIndexRange> {
//     let tree = engine.ngram(n);
//     let all = tree.is_all();
//     match all {
//         true => None,
//         false => {
//             let index_ranges = tree
//                 .ngrams()
//                 .iter()
//                 .filter_map(|ngram_index| {
//                     index_data.get_ngram_range(ngram_index).and_then(|range| {
//                         log(&format!("get ngram {:?} range {:?}", ngram_index, range));
//                         Some(NgramIndexRange {
//                             index: ngram_index.clone(),
//                             range,
//                             data: NgramData::new(),
//                         })
//                     })
//                 })
//                 .collect::<Vec<_>>();
//             Some(TreeNgramIndexRange { tree, index_ranges })
//         }
//     }
// }
