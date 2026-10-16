use super::expr::{self, Expression};
use super::functions::{self, Function, FunctionCall};
use super::loops::{self, Loop};
use super::conditional::{self, Conditional};
use super::variables::{self, Variable};
use super::commands::{self, Command};
use regex::Captures;

pub enum Statement {
    Variable(Variable),
    Function(Function),
    FunctionCall(FunctionCall),
    Loop(Loop),
    Conditional(Conditional),
    Expression(Expression),
    Command(Command),
}

pub type Block = Vec<Statement>;

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    conditional::capture(string)
        .or_else(|| loops::capture(string))
        .or_else(|| variables::capture(string))
        .or_else(|| functions::capture(string))
        .or_else(|| commands::capture(string))
        .or_else(|| expr::capture(string))
}
