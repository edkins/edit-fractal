use wasm_bindgen::prelude::*;

mod ast;
mod parse;

#[wasm_bindgen]
pub fn compile(text: &str) -> Box<[u8]> {
    let vec = match parse::parse(text) {
        Err(_) => vec![],
        Ok(e) => {
            format!("{:?}", e).as_bytes().to_vec()
        }
    };
    vec.into_boxed_slice()
}
