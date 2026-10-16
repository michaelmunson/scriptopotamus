use super::datatype::Datatype;
use super::expr::Expression;
use regex::{Regex, Captures};

pub struct Variable {
    pub name: String,
    pub datatype: Datatype,
    pub value: Option<Expression>
}

pub struct Reference {
    pub variable: Variable,
}

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let VARIABLE_PATTERN: Regex = Regex::new(r"^(?P<name>[A-Za-z_][A-Za-z0-9_]*)(?:<(?P<datatype>[^>]+)>)?=(?P<value>(?:[^=].*)?)$").unwrap();
    let REFERENCE_PATTERN: Regex = Regex::new(r"^\$(?P<reference>-{0,2}[A-Za-z_][A-Za-z0-9_-]*)$").unwrap();
    if VARIABLE_PATTERN.is_match(string) {
        Some(VARIABLE_PATTERN.captures(string).unwrap())
    } else if REFERENCE_PATTERN.is_match(string) {
        Some(REFERENCE_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}
