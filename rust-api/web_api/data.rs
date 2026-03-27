use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VecU8 {
    vec: Vec<u8>,
}

#[wasm_bindgen]
pub struct Range {
    #[wasm_bindgen(readonly)]
    pub start: u32,
    #[wasm_bindgen(readonly)]
    pub end: u32,
}

impl VecU8 {
    pub fn vec(self) -> Vec<u8> {
        self.vec
    }
}

#[wasm_bindgen]
impl VecU8 {
    #[wasm_bindgen(constructor)]
    pub fn new(global_data: Vec<u8>) -> Self {
        VecU8 { vec: global_data }
    }
}



impl From<Vec<u8>> for VecU8 {
    fn from(value: Vec<u8>) -> Self {
        VecU8 { vec: value }
    }
}