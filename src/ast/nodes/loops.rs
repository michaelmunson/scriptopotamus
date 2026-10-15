use super::expr::Expression;
use super::statements::{Block};

pub enum LoopKind {
    For,
    While,
}

pub struct Loop {
    pub kind: LoopKind,
    pub condition: Option<Expression>,
    pub body: Block,
}
