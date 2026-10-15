use super::statements::{Block};
use super::expr::Expression;
use super::datatype::{Datatype, Literal};

pub struct Parameter {
    pub name: String,
    pub datatype: Datatype,
    pub default: Option<Literal>,
    pub optional: bool,
    pub requires: Box<Parameter>,
}

pub struct Command {
    pub parent: Option<Box<Command>>,
}

pub struct Function {
    pub command: Option<Command>,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

pub struct Argument {
    pub parameter: Parameter,
    pub value: Expression
}

pub struct FunctionCall {
    pub function: Function,
    pub arguments: Vec<Argument>
}