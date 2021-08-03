use std::collections::HashMap;
use crate::ast::Expr;
use crate::dag::{Dag, DagNode, Effect, EffectType};
use crate::module_builder::{BlockType,Func,ModuleBuilder,ValType};

pub fn backend(expr_initz: Option<&Expr>, expr: &Expr, expr_escape2: &Expr, maxiter: &Expr) -> Vec<u8> {
    if expr_initz.is_none() {
        let mb = ModuleBuilder::default();
        let (mb,solve) = backend_solve(mb, expr);
        let mb = backend_main(mb, None, expr, expr_escape2, maxiter, Some(solve));
        mb.into_vec()
    } else {
        let mb = ModuleBuilder::default();
        let mb = backend_main(mb, expr_initz, expr, expr_escape2, maxiter, None);
        mb.into_vec()
    }
}

fn backend_main(mut mb: ModuleBuilder, expr_initz: Option<&Expr>, expr: &Expr, expr_escape2: &Expr, maxiter: &Expr, solve:Option<Func>) -> ModuleBuilder {
    let expr_iter = Expr::Call("+".to_owned(), vec![Expr::Var("iter".to_owned()), Expr::F64(1.0)]);
    let expr_escape1 = Expr::Call(">".to_owned(), vec![Expr::Var("iter".to_owned()), maxiter.clone()]);

    let return_thing = mb.start_func(&[ValType::F64, ValType::F64, ValType::F64, ValType::F64], &[ValType::F64]);

    let cx = mb.get_local_param(2);
    let cy = mb.get_local_param(3);
    let l0 = mb.add_local(ValType::F64);
    let l1 = mb.add_local(ValType::F64);
    let iter = mb.add_local(ValType::F64);

    let mut mb = if let Some(expr_initz) = expr_initz {
        let mut fc = FuncContext::new(mb);
        fc.env.insert("i".to_owned(), Structure::Complex(fc.dag.f64_zero(), fc.dag.f64_one()));
        fc.env.insert("c".to_owned(), Structure::Complex(fc.dag.f64_input(cx), fc.dag.f64_input(cy)));
        let initz = fc.do_expr(expr_initz);
        let mut mb = fc.done(&[Effect(EffectType::Push, initz.cx()), Effect(EffectType::Push, initz.cy())]);
        mb.local_set(l1);
        mb.local_set(l0);
        mb
    } else {
        let initzx = mb.get_local_param(0);
        let initzy = mb.get_local_param(1);
        mb.local_get(initzx);
        mb.local_get(initzy);
        mb.local_get(cx);
        mb.local_get(cy);
        mb.call(solve.unwrap());
        mb.local_set(l1);
        mb.local_set(l0);
        mb
    };

    mb.f64_const(0.0);
    mb.local_set(iter);

    mb.start_block(BlockType::Empty);
    mb.start_loop(BlockType::Empty);

    let mut fc = FuncContext::new(mb);
    fc.env.insert("i".to_owned(), Structure::Complex(fc.dag.f64_zero(), fc.dag.f64_one()));
    fc.env.insert("c".to_owned(), Structure::Complex(fc.dag.f64_input(cx), fc.dag.f64_input(cy)));
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
    mb
}

fn backend_solve(mut mb: ModuleBuilder, expr: &Expr) -> (ModuleBuilder, Func) {
    let solve = mb.start_func(&[ValType::F64, ValType::F64, ValType::F64, ValType::F64], &[ValType::F64, ValType::F64]);

    let zx = mb.get_local_param(0);
    let zy = mb.get_local_param(1);
    let cx = mb.get_local_param(2);
    let cy = mb.get_local_param(3);
    let zx1 = mb.add_local(ValType::F64);
    let zy1 = mb.add_local(ValType::F64);
    let zx2 = mb.add_local(ValType::F64);
    let zy2 = mb.add_local(ValType::F64);
    let iter = mb.add_local(ValType::I32);

    mb.i32_const(0);
    mb.local_set(iter);

    mb.start_loop(BlockType::Empty);
    mb.f64_const(1.0);
    mb.local_set(zx1);
    mb.f64_const(0.0);
    mb.local_set(zy1);
    mb.f64_const(0.0);
    mb.local_set(zx2);
    mb.f64_const(0.0);
    mb.local_set(zy2);
    let mut fc = FuncContext::new(mb);
    fc.env.insert("i".to_owned(), fc.dconst(fc.dag.f64_zero(), fc.dag.f64_one()));
    fc.env.insert("c".to_owned(), fc.dconst(fc.dag.f64_input(cx), fc.dag.f64_input(cy)));
    fc.env.insert("z".to_owned(), Structure::CxDeriv([fc.dag.f64_input(zx), fc.dag.f64_input(zy), fc.dag.f64_input(zx1), fc.dag.f64_input(zy1), fc.dag.f64_input(zx2), fc.dag.f64_input(zy2)]));
    let stuff = fc.do_expr_deriv(expr);
    let stuff1 = fc.cx_div(&stuff.derivs()[1], &stuff.derivs()[2]);
    let newz = fc.cx_sub(&Structure::Complex(fc.dag.f64_input(zx), fc.dag.f64_input(zy)), &stuff1);
    let mut mb = fc.done(&[Effect(EffectType::Push, newz.cx()), Effect(EffectType::Push, newz.cy())]);
    mb.local_set(zy);
    mb.local_set(zx);

    mb.local_get(iter);
    mb.i32_const(1);
    mb.i32_add();
    mb.local_tee(iter);

    mb.i32_const(10);
    mb.i32_lt_u();
    mb.br_if(0);

    mb.end_loop();

    mb.local_get(zx);
    mb.local_get(zy);

    mb.end_func();
    (mb, solve)
}

#[derive(Clone)]
enum Structure {
    Bool(DagNode),
    Complex(DagNode, DagNode),
    CxDeriv([DagNode;6]),
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
            _ => panic!(),
        }
    }
    fn cy(&self) -> DagNode {
        match self {
            Structure::Complex(_, y) => *y,
            _ => panic!(),
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
            _ => panic!(),
        }
    }
    fn derivs(&self) -> [Structure;3] {
        match self {
            Structure::CxDeriv(xs) => [
                Structure::Complex(xs[0], xs[1]),
                Structure::Complex(xs[2], xs[3]),
                Structure::Complex(xs[4], xs[5]),
            ],
            _ => panic!(),
        }
    }
    fn deriv(d0: Structure, d1: Structure, d2: Structure) -> Self {
        Structure::CxDeriv([d0.cx(), d0.cy(), d1.cx(), d1.cy(), d2.cx(), d2.cy()])
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

    fn dconst(&self, x: DagNode, y: DagNode) -> Structure {
        let zero = self.dag.f64_zero();
        Structure::CxDeriv([x,y,zero,zero,zero,zero])
    }

    fn do_expr_deriv(&mut self, expr: &Expr) -> Structure {
        match expr {
            Expr::F64(x) => self.dconst(self.dag.f64_const(*x), self.dag.f64_zero()),
            Expr::Var(z) => self.env.get(z).unwrap().clone(),
            Expr::Call(f, args) => {
                let d:Vec<_> = args.iter().map(|arg|self.do_expr_deriv(arg).derivs()).collect();
                match &f as &str {
                    "+" => {
                        let z0 = self.cx_add(&d[0][0], &d[1][0]);
                        let z1 = self.cx_add(&d[0][1], &d[1][1]);
                        let z2 = self.cx_add(&d[0][2], &d[1][2]);
                        Structure::deriv(z0, z1, z2)
                    }
                    "-" => {
                        let z0 = self.cx_sub(&d[0][0], &d[1][0]);
                        let z1 = self.cx_sub(&d[0][1], &d[1][1]);
                        let z2 = self.cx_sub(&d[0][2], &d[1][2]);
                        Structure::deriv(z0, z1, z2)
                    }
                    "*" => {
                        let [a,b,c] = d[0].clone();
                        let [d,e,f] = d[1].clone();
                        let ad = self.cx_mul(&a,&d);
                        let ae = self.cx_mul(&a,&e);
                        let bd = self.cx_mul(&b,&d);
                        let cd = self.cx_mul(&c,&d);
                        let be = self.cx_mul(&b,&e);
                        let af = self.cx_mul(&a,&f);
                        let z1 = self.cx_add(&ae, &bd);
                        let be2 = self.cx_add(&be, &be);
                        let be2_af = self.cx_add(&be2, &af);
                        let z2 = self.cx_add(&cd, &be2_af);
                        Structure::deriv(ad, z1, z2)
                    }
                    "/" => {
                        panic!();
                    }
                    "neg" => {
                        let z0 = self.cx_neg(&d[0][0]);
                        let z1 = self.cx_neg(&d[0][1]);
                        let z2 = self.cx_neg(&d[0][2]);
                        Structure::deriv(z0, z1, z2)
                    }
                    _ => panic!()
                }
            }
        }
    }

    fn cx_add(&mut self, a: &Structure, b: &Structure) -> Structure {
        let x = self.dag.f64_add(a.cx(), b.cx());
        let y = self.dag.f64_add(a.cy(), b.cy());
        Structure::Complex(x, y)
    }

    fn cx_sub(&mut self, a: &Structure, b: &Structure) -> Structure {
        let x = self.dag.f64_sub(a.cx(), b.cx());
        let y = self.dag.f64_sub(a.cy(), b.cy());
        Structure::Complex(x, y)
    }

    fn cx_mul(&mut self, z0: &Structure, z1: &Structure) -> Structure {
        let x0_x1 = self.dag.f64_mul(z0.cx(), z1.cx());
        let x0_y1 = self.dag.f64_mul(z0.cx(), z1.cy());
        let x1_y0 = self.dag.f64_mul(z1.cx(), z0.cy());
        let y0_y1 = self.dag.f64_mul(z0.cy(), z1.cy());
        let x = self.dag.f64_sub(x0_x1, y0_y1);
        let y = self.dag.f64_add(x0_y1, x1_y0);
        Structure::Complex(x, y)
    }

    fn cx_div(&mut self, z0: &Structure, z1: &Structure) -> Structure {
        let ac = self.dag.f64_mul(z0.cx(), z1.cx());
        let bd = self.dag.f64_mul(z0.cy(), z1.cy());
        let bc = self.dag.f64_mul(z0.cy(), z1.cx());
        let ad = self.dag.f64_mul(z0.cx(), z1.cy());
        let cc = self.dag.f64_mul(z1.cx(), z1.cx());
        let dd = self.dag.f64_mul(z1.cy(), z1.cy());
        let rr = self.dag.f64_add(cc, dd);
        let xrr = self.dag.f64_add(ac, bd);
        let yrr = self.dag.f64_sub(bc, ad);
        let x = self.dag.f64_div(xrr, rr);
        let y = self.dag.f64_div(yrr, rr);
        Structure::Complex(x, y)
    }

    fn cx_neg(&mut self, z: &Structure) -> Structure {
        let x = self.dag.f64_neg(z.cx());
        let y = self.dag.f64_neg(z.cy());
        Structure::Complex(x, y)
    }

    fn do_expr(&mut self, expr: &Expr) -> Structure {
        match expr {
            Expr::F64(x) => Structure::Complex(self.dag.f64_const(*x), self.dag.f64_const(0.0)),
            Expr::Var(z) => self.env.get(z).unwrap().clone(),
            Expr::Call(f, args) => {
                let structs:Vec<_> = args.iter().map(|arg|self.do_expr(arg)).collect();
                match &f as &str {
                    "+" => self.cx_add(&structs[0], &structs[1]),
                    "-" => self.cx_sub(&structs[0], &structs[1]),
                    "*" => self.cx_mul(&structs[0], &structs[1]),
                    "/" => self.cx_div(&structs[0], &structs[1]),
                    "neg" => self.cx_neg(&structs[0]),
                    "sqabs" => {
                        let xx = self.dag.f64_mul(structs[0].cx(), structs[0].cx());
                        let yy = self.dag.f64_mul(structs[0].cy(), structs[0].cy());
                        let rr = self.dag.f64_add(xx, yy);
                        Structure::Complex(rr, self.dag.f64_zero())
                    }
                    "real" => {
                        Structure::Complex(structs[0].cx(), self.dag.f64_zero())
                    }
                    "conj" => {
                        let y = self.dag.f64_neg(structs[0].cy());
                        Structure::Complex(structs[0].cx(), y)
                    }
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
