use wasm_bindgen::prelude::*;

mod ast;
mod backend;
mod dag;
mod module_builder;
mod parse;

#[wasm_bindgen]
pub fn compile(texts: Box<[JsValue]>) -> Box<[u8]> {
    if texts[0].as_string().unwrap().is_empty() {
        let exprs:Vec<_> = texts[1..].iter().map(|text|parse::parse(&text.as_string().unwrap()).unwrap()).collect();
        backend::backend(None, &exprs[0], &exprs[1], &exprs[2]).into_boxed_slice()
    } else {
        let exprs:Vec<_> = texts.iter().map(|text|parse::parse(&text.as_string().unwrap()).unwrap()).collect();
        backend::backend(Some(&exprs[0]), &exprs[1], &exprs[2], &exprs[3]).into_boxed_slice()
    }
}
