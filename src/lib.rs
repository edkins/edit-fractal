use wasm_bindgen::prelude::*;

mod ast;
mod backend;
mod dag;
mod module_builder;
mod parse;

#[wasm_bindgen]
pub fn compile(texts: Box<[JsValue]>) -> Box<[u8]> {
    let exprs:Vec<_> = texts.iter().map(|text|parse::parse(&text.as_string().unwrap()).unwrap()).collect();
    backend::backend(&exprs[0], &exprs[1]).into_boxed_slice()
}
