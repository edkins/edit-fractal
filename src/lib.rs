use wasm_bindgen::prelude::*;

mod ast;
mod backend;
mod dag;
mod module_builder;
mod parse;

#[wasm_bindgen]
pub fn compile(text: &str) -> Box<[u8]> {
    match parse::parse(text) {
        Err(e) => panic!("{:?}", e),
        Ok(expr) => {
            backend::backend(&expr).into_boxed_slice()
        }
    }
}
