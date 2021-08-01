#[derive(Debug, Clone)]
pub enum Expr {
    F64(f64),
    Var(String),
    Call(String, Vec<Expr>),
}
