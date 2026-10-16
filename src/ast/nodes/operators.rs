use regex::{Regex, Captures};

pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Power,
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Echo
}

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let OPERATOR_PATTERN: Regex = Regex::new(r"^(?P<operator>\*\*|==|!=|>=|<=|->|[-+*/%><])$").unwrap();
    OPERATOR_PATTERN.captures(string)
}
