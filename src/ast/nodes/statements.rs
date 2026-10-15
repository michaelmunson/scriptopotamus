use super::expr::Expression;
use super::functions::{Function, FunctionCall};
use super::loops::Loop;
use super::conditional::Conditional;
use super::variables::Variable;

pub enum Statement {
    Variable(Variable),
    Function(Function),
    FunctionCall(FunctionCall),
    Loop(Loop),
    Conditional(Conditional),
    Expression(Expression),
}

pub type Block = Vec<Statement>;