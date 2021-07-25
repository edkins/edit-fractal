use crate::ast::Expr;
use crate::module_builder::{ModuleBuilder,ValType};

pub fn backend(expr: &Expr) -> Vec<u8> {
    let mut mb = ModuleBuilder::default();
    let return_thing = mb.start_func(&[], &[ValType::F32], &[]);
    mb.do_expr(expr);
    mb.end_func();
    mb.export_func(return_thing, "return_thing");
    mb.into_vec()
}

impl ModuleBuilder {
    fn do_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::F32(x) => self.f32_const(*x),
            Expr::Call(f, args) => {
                for arg in args {
                    self.do_expr(arg);
                }
                match &f as &str {
                    "+" => self.f32_add(),
                    _ => panic!("Cannot call {}", f),
                }
            }
        }
    }
}
