use crate::{
    ngram::data::FromToData,
    web_api::index::{NgramIndex, NgramIndexVec},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SearchEngine {
    engine: crate::ngram::search::SearchEngine,
}

#[wasm_bindgen]
pub struct SearchOneEngine {
    engine: crate::ngram::search::SearchOneEngine,
}

#[wasm_bindgen]
impl SearchEngine {
    pub fn search(&self, pattern: &str) -> Result<SearchOneEngine, JsValue> {
        self.engine
            .search(pattern)
            .map(SearchOneEngine::from)
            .map_err(|e| JsValue::from_str(&format!("search error: {}", e)))
    }
    #[wasm_bindgen(constructor)]
    pub fn new(global_data: Vec<u8>) -> Result<SearchEngine, JsValue> {
        let data = crate::ngram::data::GlobalData::from_data(&global_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse global data: {}", e)))?;
        let engine = crate::ngram::search::SearchEngine::from(data);
        Ok(SearchEngine { engine })
    }
}

#[wasm_bindgen]
impl SearchOneEngine {
    pub fn ngrams(&self) -> NgramIndexVec {
        NgramIndexVec::from(
            self.engine
                .ngrams()
                .0
                .into_iter()
                .map(NgramIndex::from)
                .collect::<Vec<NgramIndex>>(),
        )
    }
}

impl From<crate::ngram::search::SearchEngine> for SearchEngine {
    fn from(value: crate::ngram::search::SearchEngine) -> Self {
        SearchEngine { engine: value }
    }
}

impl From<crate::ngram::search::SearchOneEngine> for SearchOneEngine {
    fn from(value: crate::ngram::search::SearchOneEngine) -> Self {
        SearchOneEngine { engine: value }
    }
}
