use super::expr::Expression;
use super::statements::{Block};
use regex::{Regex, Captures};

pub enum LoopKind {
    For,
    While,
}

pub struct Loop {
    pub kind: LoopKind,
    pub condition: Option<Expression>,
    pub body: Block,
}

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let LOOP_PATTERN: Regex = Regex::new(r"^(?P<kind>for|while)(?:\s+(?P<condition>.+))?$").unwrap();
    LOOP_PATTERN.captures(string)
}
