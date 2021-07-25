#[derive(Debug, Clone)]
pub enum Expr {
    F32(f32),
    Call(String, Vec<Expr>),
}
