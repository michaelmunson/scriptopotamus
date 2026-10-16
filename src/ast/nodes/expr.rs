use super::datatype::{Literal};
use super::operators::Operator;
use super::functions::FunctionCall;
use super::variables::{Reference};
use regex::{Regex, Captures};

pub enum ExpressionPart {
    Reference(Reference),
    Literal(Literal),
    Operator(Operator),
    FunctionCall(FunctionCall),
}

pub type Expression = Vec<ExpressionPart>;

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let REFERENCE_PATTERN: Regex = Regex::new(r"^\s*(?P<reference>\$-{0,2}[A-Za-z_][A-Za-z0-9_-]*)\s*(?P<rest>.*)$").unwrap();
    let LITERAL_PATTERN: Regex = Regex::new(r#"^\s*(?P<literal>"[^"]*"|'[^']*'|-?\d+\.\d+|-?\d+|(?:true|false)\b)\s*(?P<rest>.*)$"#).unwrap();
    let FLAG_PATTERN: Regex = Regex::new(r"^\s*(?P<flag>--[A-Za-z_][A-Za-z0-9_-]*)\s*(?P<rest>.*)$").unwrap();
    let OPERATOR_PATTERN: Regex = Regex::new(r"^\s*(?P<operator>\*\*|==|!=|>=|<=|->|[-+*/%><])\s*(?P<rest>.*)$").unwrap();
    let NAME_PATTERN: Regex = Regex::new(r"^\s*(?P<name>[A-Za-z_][A-Za-z0-9_-]*)\s*(?P<rest>.*)$").unwrap();
    if REFERENCE_PATTERN.is_match(string) {
        Some(REFERENCE_PATTERN.captures(string).unwrap())
    } else if LITERAL_PATTERN.is_match(string) {
        Some(LITERAL_PATTERN.captures(string).unwrap())
    } else if FLAG_PATTERN.is_match(string) {
        Some(FLAG_PATTERN.captures(string).unwrap())
    } else if OPERATOR_PATTERN.is_match(string) {
        Some(OPERATOR_PATTERN.captures(string).unwrap())
    } else if NAME_PATTERN.is_match(string) {
        Some(NAME_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}
