use std::collections::HashMap;
use crate::module_builder::{Local,ModuleBuilder,ValType};

#[derive(Clone,Copy,Hash,Eq,PartialEq)]
pub enum DagNode {
    F64(u64),  // need to store f64 as bits in order for Eq to work
    Node(usize),
    Input(Local),
}

#[derive(Clone,Copy,Hash,Eq,PartialEq)]
enum DagCalc {
    F64Neg(DagNode),
    F64Add(DagNode, DagNode),
    F64Sub(DagNode, DagNode),
    F64Mul(DagNode, DagNode),
    F64Div(DagNode, DagNode),
    F64Lt(DagNode, DagNode),
    F64Gt(DagNode, DagNode),
    F64Le(DagNode, DagNode),
    F64Ge(DagNode, DagNode),
}

#[derive(Default)]
pub struct Dag {
    memo: HashMap<DagCalc, DagNode>,
    nodes: Vec<DagCalc>,
}

pub struct Effect(pub EffectType, pub DagNode);

pub enum EffectType {
    BrIf(usize),
    Push,
}

impl EffectType {
    fn emit(&self, mb: &mut ModuleBuilder) {
        match self {
            EffectType::BrIf(label) => mb.br_if(*label),
            EffectType::Push => {}
        }
    }
}

impl DagCalc {
    fn dependencies(&self) -> Vec<DagNode> {
        match self {
            DagCalc::F64Neg(x) => vec![*x],
            DagCalc::F64Add(x,y) | DagCalc::F64Sub(x,y) | DagCalc::F64Mul(x,y) | DagCalc::F64Div(x,y)
                | DagCalc::F64Lt(x,y) | DagCalc::F64Gt(x,y) | DagCalc::F64Le(x,y) | DagCalc::F64Ge(x,y) => vec![*x,*y],
        }
    }
}

impl DagNode {
    pub fn is_const_zero(self) -> bool {
        match self {
            DagNode::F64(x) => f64::from_bits(x) == 0.0,
            _ => false,
        }
    }

    fn is_const_one(self) -> bool {
        match self {
            DagNode::F64(x) => f64::from_bits(x) == 1.0,
            _ => false,
        }
    }
}

impl Dag {
    pub fn f64_zero(&self) -> DagNode {
        self.f64_const(0.0)
    }

    pub fn f64_const(&self, x: f64) -> DagNode {
        DagNode::F64(x.to_bits())
    }

    pub fn f64_input(&self, local: Local) -> DagNode {
        DagNode::Input(local)
    }

    pub fn f64_neg(&mut self, x: DagNode) -> DagNode {
        match x {
            DagNode::F64(a) => DagNode::F64((-f64::from_bits(a)).to_bits()),
            _ => self.calc(DagCalc::F64Neg(x)),
        }
    }

    pub fn f64_add(&mut self, x: DagNode, y: DagNode) -> DagNode {
        match (x,y) {
            (DagNode::F64(a), DagNode::F64(b)) => DagNode::F64((f64::from_bits(a) + f64::from_bits(b)).to_bits()),
            _ => {
                if x.is_const_zero() {
                    y
                } else if y.is_const_zero() {
                    x
                } else {
                    self.calc(DagCalc::F64Add(x, y))
                }
            }
        }
    }

    pub fn f64_sub(&mut self, x: DagNode, y: DagNode) -> DagNode {
        match (x,y) {
            (DagNode::F64(a), DagNode::F64(b)) => DagNode::F64((f64::from_bits(a) - f64::from_bits(b)).to_bits()),
            _ => {
                if x.is_const_zero() {
                    self.f64_neg(y)
                } else if y.is_const_zero() {
                    x
                } else {
                    self.calc(DagCalc::F64Sub(x, y))
                }
            }
        }
    }

    pub fn f64_mul(&mut self, x: DagNode, y: DagNode) -> DagNode {
        match (x,y) {
            (DagNode::F64(a), DagNode::F64(b)) => DagNode::F64((f64::from_bits(a) * f64::from_bits(b)).to_bits()),
            _ => {
                if x.is_const_zero() || y.is_const_zero() {
                    DagNode::F64(0.0f64.to_bits())
                } else if x.is_const_one() {
                    y
                } else if y.is_const_one() {
                    x
                } else {
                    self.calc(DagCalc::F64Mul(x, y))
                }
            }
        }
    }

    pub fn f64_div(&mut self, x: DagNode, y: DagNode) -> DagNode {
        match (x,y) {
            (DagNode::F64(a), DagNode::F64(b)) => DagNode::F64((f64::from_bits(a) / f64::from_bits(b)).to_bits()),
            _ => {
                if x.is_const_zero() {
                    DagNode::F64(0.0f64.to_bits())
                } else if y.is_const_one() {
                    x
                } else {
                    self.calc(DagCalc::F64Div(x, y))
                }
            }
        }
    }

    pub fn f64_lt(&mut self, x: DagNode, y: DagNode) -> DagNode {
        self.calc(DagCalc::F64Lt(x, y))
    }

    pub fn f64_gt(&mut self, x: DagNode, y: DagNode) -> DagNode {
        self.calc(DagCalc::F64Gt(x, y))
    }

    pub fn f64_le(&mut self, x: DagNode, y: DagNode) -> DagNode {
        self.calc(DagCalc::F64Le(x, y))
    }

    pub fn f64_ge(&mut self, x: DagNode, y: DagNode) -> DagNode {
        self.calc(DagCalc::F64Ge(x, y))
    }

    fn calc(&mut self, calc: DagCalc) -> DagNode {
        if let Some(n) = self.memo.get(&calc) {
            return *n;
        }
        let n = DagNode::Node(self.nodes.len());
        self.nodes.push(calc);
        self.memo.insert(calc, n);
        n
    }

    fn dependencies(&self, node: DagNode) -> Vec<DagNode> {
        match node {
            DagNode::Node(i) => self.nodes[i].dependencies(),
            DagNode::F64(_) | DagNode::Input(_) => vec![],
        }
    }

    fn add_usage(&self, result: &mut HashMap<DagNode, usize>, node: DagNode) {
        if let Some(x) = result.get_mut(&node) {
            *x += 1;
        } else {
            result.insert(node, 1);
            for n in &self.dependencies(node) {
                self.add_usage(result, *n);
            }
        }
    }

    fn usage(&self, nodes: impl Iterator<Item=DagNode>) -> HashMap<DagNode, usize> {
        let mut result = HashMap::new();
        for node in nodes {
            self.add_usage(&mut result, node);
        }
        result
    }

    fn emit_recursive(&self, mb: &mut ModuleBuilder, placement: &mut HashMap<DagNode, Local>, usage: &HashMap<DagNode, usize>, node: DagNode) {
        if let Some(local) = placement.get(&node) {
            mb.local_get(*local);
        } else {
            match node {
                DagNode::F64(x) => mb.f64_const(f64::from_bits(x)),
                DagNode::Input(local) => mb.local_get(local),
                DagNode::Node(i) => {
                    let typ;
                    match self.nodes[i] {
                        DagCalc::F64Neg(x) => {
                            self.emit_recursive(mb, placement, usage, x);
                            typ = ValType::F64;
                        }
                        DagCalc::F64Add(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_add();
                            typ = ValType::F64;
                        }
                        DagCalc::F64Sub(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_sub();
                            typ = ValType::F64;
                        }
                        DagCalc::F64Mul(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_mul();
                            typ = ValType::F64;
                        }
                        DagCalc::F64Div(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_div();
                            typ = ValType::F64;
                        }
                        DagCalc::F64Lt(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_lt();
                            typ = ValType::I32;
                        }
                        DagCalc::F64Gt(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_gt();
                            typ = ValType::I32;
                        }
                        DagCalc::F64Le(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_le();
                            typ = ValType::I32;
                        }
                        DagCalc::F64Ge(x,y) => {
                            self.emit_recursive(mb, placement, usage, x);
                            self.emit_recursive(mb, placement, usage, y);
                            mb.f64_ge();
                            typ = ValType::I32;
                        }
                    }
                    if let Some(x) = usage.get(&node) {
                        if *x > 1 {
                            let local = mb.add_local(typ);
                            mb.local_tee(local);
                            placement.insert(node, local);
                        }
                    }
                }
            }
        }
    }

    pub fn emit(self, mb: &mut ModuleBuilder, effects: &[Effect]) {
        let usage = self.usage(effects.iter().map(|e|e.1));
        let mut placement = HashMap::new();
        for effect in effects {
            self.emit_recursive(mb, &mut placement, &usage, effect.1);
            effect.0.emit(mb);
        }
    }
}


