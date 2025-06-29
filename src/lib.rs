pub mod config;
pub mod index_builder;
pub mod index_file;
pub mod index_regex;
use crate::index_file::{FromToData, IndexData};
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
}

#[wasm_bindgen]
pub fn greet(a: &str) {
    let a = format!("aaa {}", a);
    unsafe {
        alert("asd");
    }
}

// #[wasm_bindgen]
// pub fn build_index_data(data: Vec<u8>, n: usize) -> IndexData {
//     index_file::IndexData::from_data(data).unwrap()
// }

// #[wasm_bindgen]
// pub fn index_regex_engen(pattern: &str, n: usize) -> (bool, String) {
//     let engine = index_regex::Engine::new(pattern).unwrap();
//     let tree = engine.ngram(4);
// }
