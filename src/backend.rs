use std::collections::HashMap;
use crate::ast::Expr;
use crate::dag::{Dag, DagNode, Effect, EffectType};
use crate::module_builder::{ModuleBuilder,ValType};

pub fn backend(expr: &Expr) -> Vec<u8> {
    let mut mb = ModuleBuilder::default();
    let return_thing = mb.start_func(&[], &[ValType::F64]);

    let l0 = mb.add_local(ValType::F64);
    let l1 = mb.add_local(ValType::F64);
    mb.f64_const(420.0);
    mb.local_set(l0);
    mb.f64_const(421.0);
    mb.local_set(l1);

    let mut fc = FuncContext::new(mb);
    fc.env.insert("z".to_owned(), Structure::Complex(fc.dag.f64_input(l0), fc.dag.f64_input(l1)));
    let newz = fc.do_expr(expr);
    let mut mb = fc.done(&[Effect(EffectType::Push, newz.cx()), Effect(EffectType::Push, newz.cy())]);
    mb.local_set(l1);
    mb.local_set(l0);
    mb.f64_const(42.0);

    mb.end_func();
    mb.export_func(return_thing, "return_thing");
    mb.into_vec()
}

#[derive(Clone)]
enum Structure {
    Complex(DagNode, DagNode),
}

struct FuncContext {
    mb: ModuleBuilder,
    dag: Dag,
    env: HashMap<String, Structure>,
}

impl Structure {
    fn cx(&self) -> DagNode {
        match self {
            Structure::Complex(x, _) => *x
        }
    }
    fn cy(&self) -> DagNode {
        match self {
            Structure::Complex(_, y) => *y
        }
    }
}

impl FuncContext {
    fn new(mb: ModuleBuilder) -> Self {
        FuncContext {
            mb,
            dag: Dag::default(),
            env: HashMap::new(),
        }
    }

    fn done(mut self, effects: &[Effect]) -> ModuleBuilder {
        self.dag.emit(&mut self.mb, effects);
        self.mb
    }

    fn do_expr(&mut self, expr: &Expr) -> Structure {
        match expr {
            Expr::F64(x) => Structure::Complex(self.dag.f64_const(*x), self.dag.f64_const(0.0)),
            Expr::Var(z) => self.env.get(z).unwrap().clone(),
            Expr::Call(f, args) => {
                let structs:Vec<_> = args.iter().map(|arg|self.do_expr(arg)).collect();
                match &f as &str {
                    "+" => {
                        let x = self.dag.f64_add(structs[0].cx(), structs[1].cx());
                        let y = self.dag.f64_add(structs[0].cy(), structs[1].cy());
                        Structure::Complex(x, y)
                    }
                    "-" => {
                        let x = self.dag.f64_sub(structs[0].cx(), structs[1].cx());
                        let y = self.dag.f64_sub(structs[0].cy(), structs[1].cy());
                        Structure::Complex(x, y)
                    }
                    "*" => {
                        let x0_x1 = self.dag.f64_mul(structs[0].cx(), structs[1].cx());
                        let x0_y1 = self.dag.f64_mul(structs[0].cx(), structs[1].cy());
                        let x1_y0 = self.dag.f64_mul(structs[1].cx(), structs[0].cy());
                        let y0_y1 = self.dag.f64_mul(structs[0].cy(), structs[1].cy());
                        let x = self.dag.f64_sub(x0_x1, y0_y1);
                        let y = self.dag.f64_add(x0_y1, x1_y0);
                        Structure::Complex(x, y)
                    }
                    "/" => panic!("Division not implemented yet"),
                    _ => panic!("Cannot call {}", f),
                }
            }
        }
    }
}
