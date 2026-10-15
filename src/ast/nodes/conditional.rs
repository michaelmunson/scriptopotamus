use super::expr::Expression;
use super::statements::{Block};
use super::datatype::{Pattern};

pub struct IfElse {
    pub condition: Option<Expression>,
    pub body:Block,
    pub alternate: Option<Box<IfElse>>,
}

pub struct Case {
    pub value: Expression,
    pub items: Vec<CaseItem>,
}

pub enum CaseTerminator {
    Break,
    Continue,
    Fallthrough,
}

pub struct CaseItem {
    pub pattern: Box<Pattern>,
    pub body: Block,
    pub terminator: CaseTerminator,
}

pub enum Conditional {
    IfElse(IfElse),
    Case(Case),
}