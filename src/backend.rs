use std::collections::HashMap;
use crate::ast::Expr;
use crate::dag::{Dag, DagNode, Effect, EffectType};
use crate::module_builder::{BlockType,ModuleBuilder,ValType};

pub fn backend(expr: &Expr) -> Vec<u8> {
    let expr_iter = Expr::Call("+".to_owned(), vec![Expr::Var("iter".to_owned()), Expr::F64(1.0)]);
    let expr_escape1 = Expr::Call(">".to_owned(), vec![Expr::Var("iter".to_owned()), Expr::F64(100.0)]);
    let expr_escape2 = Expr::Call(
        ">".to_owned(),
        vec![
            Expr::Call("sqabs".to_owned(), vec![Expr::Var("z".to_owned())]),
            Expr::F64(4.0)]);

    let mut mb = ModuleBuilder::default();
    let return_thing = mb.start_func(&[], &[ValType::F64]);

    let l0 = mb.add_local(ValType::F64);
    let l1 = mb.add_local(ValType::F64);
    let iter = mb.add_local(ValType::F64);
    mb.f64_const(0.75);
    mb.local_set(l0);
    mb.f64_const(0.75);
    mb.local_set(l1);
    mb.f64_const(0.0);
    mb.local_set(iter);

    mb.start_block(BlockType::Empty);
    mb.start_loop(BlockType::Empty);

    let mut fc = FuncContext::new(mb);
    fc.env.insert("z".to_owned(), Structure::Complex(fc.dag.f64_input(l0), fc.dag.f64_input(l1)));
    fc.env.insert("iter".to_owned(), Structure::Complex(fc.dag.f64_input(iter), fc.dag.f64_zero()));
    let escape1 = fc.do_expr(&expr_escape1);
    let escape2 = fc.do_expr(&expr_escape2);
    let newz = fc.do_expr(expr);
    let newiter = fc.do_expr(&expr_iter);
    let mut mb = fc.done(&[
                         Effect(EffectType::BrIf(1), escape1.boolean()),
                         Effect(EffectType::BrIf(1), escape2.boolean()),
                         Effect(EffectType::Push, newz.cx()),
                         Effect(EffectType::Push, newz.cy()),
                         Effect(EffectType::Push, newiter.as_real_f64())]);
    mb.local_set(iter);
    mb.local_set(l1);
    mb.local_set(l0);

    mb.br(0);
    mb.end_loop();
    mb.end_block();
    mb.local_get(iter);

    mb.end_func();
    mb.export_func(return_thing, "return_thing");
    mb.into_vec()
}

#[derive(Clone)]
enum Structure {
    Bool(DagNode),
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
            Structure::Complex(x, _) => *x,
            Structure::Bool(_) => panic!(),
        }
    }
    fn cy(&self) -> DagNode {
        match self {
            Structure::Complex(_, y) => *y,
            Structure::Bool(_) => panic!(),
        }
    }
    fn as_real_f64(&self) -> DagNode {
        if !self.cy().is_const_zero() {
            panic!();
        }
        self.cx()
    }
    fn boolean(&self) -> DagNode {
        match self {
            Structure::Bool(b) => *b,
            Structure::Complex(_, _) => panic!(),
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
                    "sqabs" => {
                        let xx = self.dag.f64_mul(structs[0].cx(), structs[0].cx());
                        let yy = self.dag.f64_mul(structs[0].cy(), structs[0].cy());
                        let rr = self.dag.f64_add(xx, yy);
                        Structure::Complex(rr, self.dag.f64_zero())
                    }
                    "real" => {
                        Structure::Complex(structs[0].cx(), self.dag.f64_zero())
                    }
                    "/" => panic!("Division not implemented yet"),
                    "<" => Structure::Bool(self.dag.f64_lt(structs[0].as_real_f64(), structs[1].as_real_f64())),
                    ">" => Structure::Bool(self.dag.f64_gt(structs[0].as_real_f64(), structs[1].as_real_f64())),
                    "<=" => Structure::Bool(self.dag.f64_le(structs[0].as_real_f64(), structs[1].as_real_f64())),
                    ">=" => Structure::Bool(self.dag.f64_ge(structs[0].as_real_f64(), structs[1].as_real_f64())),
                    _ => panic!("Cannot call {}", f),
                }
            }
        }
    }
}
