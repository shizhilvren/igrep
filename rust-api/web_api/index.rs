use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NgramIndexVec {
    vec: Vec<NgramIndex>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NgramIndex {
    ngram: crate::ngram::index::NgramIndex,
}

// #[wasm_bindgen]
// impl NgramIndexVec{
//     pub fn vec(&self) -> Vec<NgramIndex> {
//         self.vec.clone()
//     }
// }

// #[wasm_bindgen]
// impl NgramIndex {
//     pub fn ngrams(&self) -> Vec<u8> {
//         self.ngram.ngrams().to_vec()
//     }
// }

impl From<crate::ngram::index::NgramIndex> for NgramIndex {
    fn from(value: crate::ngram::index::NgramIndex) -> Self {
        NgramIndex { ngram: value }
    }
}

impl From<Vec<NgramIndex>> for NgramIndexVec {
    fn from(value: Vec<NgramIndex>) -> Self {
        NgramIndexVec { vec: value }
    }
}
