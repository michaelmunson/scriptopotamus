use super::statements::{Block};
use super::expr::Expression;
use super::datatype::{Datatype, Literal};
use regex::{Regex, Captures};

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

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let FUNCTION_PATTERN: Regex = Regex::new(r"^(?P<command>[A-Za-z0-9_.-]*\.)?(?P<name>[A-Za-z_][A-Za-z0-9_-]*)\((?P<parameters>[^)]*)\)?$").unwrap();
    let OPTIONAL_PARAMETER_PATTERN: Regex = Regex::new(r"^\[(?P<optional>-{0,2}[A-Za-z_][A-Za-z0-9_-]*)(?:<(?P<datatype>[^>]+)>)?(?:=(?P<default>[^\]]*))?\]$").unwrap();
    let PARAMETER_PATTERN: Regex = Regex::new(r"^(?P<parameter>-{0,2}[A-Za-z_][A-Za-z0-9_-]*)<(?P<datatype>[^>]+)>(?:=(?P<default>.*))?$").unwrap();
    let CALL_PATTERN: Regex = Regex::new(r"^(?P<call>[A-Za-z_][A-Za-z0-9_-]*)(?:\s+(?P<arguments>.+))?$").unwrap();
    if FUNCTION_PATTERN.is_match(string) {
        Some(FUNCTION_PATTERN.captures(string).unwrap())
    } else if OPTIONAL_PARAMETER_PATTERN.is_match(string) {
        Some(OPTIONAL_PARAMETER_PATTERN.captures(string).unwrap())
    } else if PARAMETER_PATTERN.is_match(string) {
        Some(PARAMETER_PATTERN.captures(string).unwrap())
    } else if CALL_PATTERN.is_match(string) {
        Some(CALL_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}
