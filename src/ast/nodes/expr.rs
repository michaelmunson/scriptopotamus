use super::datatype::{Literal};
use super::operators::Operator;
use super::functions::FunctionCall;
use super::variables::{Reference};

pub enum ExpressionPart {
    Reference(Reference),
    Literal(Literal),
    Operator(Operator),
    FunctionCall(FunctionCall),
}

pub type Expression = Vec<ExpressionPart>;