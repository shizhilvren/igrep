use crate::{
    ngram::data::FromToData,
    web_api::{
        data::{Range, VecU8},
        index::{FileIndex, NgramIndex, NgramIndexVec},
    },
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
pub struct SearchOneFilesLinesStructResult {
    result: crate::ngram::search::SearchOneFilesLinesStructResult,
}

#[wasm_bindgen]
pub struct SearchOneFileLinesContentResult {
    result: crate::ngram::search::SearchOneFileLinesContentResult,
}

#[wasm_bindgen]
pub struct SearchOneLineContentResult {
    result: crate::ngram::search::SearchOneLineContentResult,
}

#[wasm_bindgen]
impl SearchOneLineContentResult {
    pub fn line_num(&self) -> u32 {
        self.result.line_num()
    }
    pub fn content(&self) -> String {
        self.result.content().clone()
    }
    pub fn match_range(&self) -> Vec<Range> {
        self.result
            .match_range()
            .iter()
            .map(|r| Range {
                start: r.0,
                end: r.1,
            })
            .collect()
    }
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
    pub fn new(global_data: VecU8) -> Result<SearchEngine, JsValue> {
        let data = crate::ngram::data::GlobalData::from_data(&global_data.vec())
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
    pub fn files_lines(
        &self,
        ngrams_index: NgramIndexVec,
        datas: Vec<VecU8>,
    ) -> Result<SearchOneFilesLinesStructResult, JsValue> {
        self.engine
            .files_lines(
                crate::ngram::index::NgramIndexVec::from(
                    ngrams_index
                        .vec()
                        .into_iter()
                        .map(|ng| ng.into())
                        .collect::<Vec<_>>(),
                ),
                datas.into_iter().map(|d| d.vec()).collect::<Vec<_>>(),
            )
            .map(SearchOneFilesLinesStructResult::from)
            .map_err(|e| JsValue::from_str(&format!("files_lines error: {}", e)))
    }
    pub fn file_lines_match(
        &self,
        file_index: FileIndex,
        file_data: VecU8,
        result: &SearchOneFilesLinesStructResult,
    ) -> Result<SearchOneFileLinesContentResult, JsValue> {
        self.engine
            .file_lines_match(file_index.into(), file_data.vec(), &result.result)
            .map(SearchOneFileLinesContentResult::from)
            .map_err(|e| JsValue::from_str(&format!("file_lines_match error: {}", e)))
    }
}

#[wasm_bindgen]
impl SearchOneFilesLinesStructResult {
    pub fn files(&self) -> Result<Vec<FileIndex>, JsValue> {
        self.result
            .files()
            .and_then(|v| Ok(v.into_iter().map(FileIndex::from).collect::<Vec<_>>()))
            .map_err(|e| JsValue::from_str(&format!("files error: {}", e)))
    }
}

#[wasm_bindgen]
impl SearchOneFileLinesContentResult {
    pub fn lines(&self) -> Vec<SearchOneLineContentResult> {
        self.result
            .lines()
            .into_iter()
            .map(|v| SearchOneLineContentResult::from(v.clone()))
            .collect::<Vec<_>>()
    }
    pub fn full_file_name(&self) -> String {
        self.result.full_file_name().clone()
    }
    pub fn is_empty(&self) -> bool {
        self.result.is_empty()
    }   
}

impl From<crate::ngram::search::SearchOneFilesLinesStructResult>
    for SearchOneFilesLinesStructResult
{
    fn from(value: crate::ngram::search::SearchOneFilesLinesStructResult) -> Self {
        SearchOneFilesLinesStructResult { result: value }
    }
}

impl From<crate::ngram::search::SearchOneFileLinesContentResult>
    for SearchOneFileLinesContentResult
{
    fn from(value: crate::ngram::search::SearchOneFileLinesContentResult) -> Self {
        SearchOneFileLinesContentResult { result: value }
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

impl From<crate::ngram::search::SearchOneLineContentResult> for SearchOneLineContentResult {
    fn from(value: crate::ngram::search::SearchOneLineContentResult) -> Self {
        SearchOneLineContentResult { result: value }
    }
}
