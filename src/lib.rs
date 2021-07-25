use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(text: &str) -> Box<[u8]> {
    b"hello".to_vec().into_boxed_slice()
}
