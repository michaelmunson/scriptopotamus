pub mod expr;
pub mod param;
pub mod stmt;
pub mod types;

pub use expr::*;
pub use param::*;
pub use stmt::*;
pub use types::*;

#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Stmt>,
}
