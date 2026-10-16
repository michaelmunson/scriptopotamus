use super::expr::Expression;
use super::statements::{Block};
use super::datatype::{Pattern};
use regex::{Regex, Captures};

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

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let IF_PATTERN: Regex = Regex::new(r"^(?P<keyword>if|elif)\s+(?P<condition>[^;]+?)\s*(?:;\s*(?P<body>.*))?$").unwrap();
    let ELSE_PATTERN: Regex = Regex::new(r"^(?P<keyword>else)(?:;?\s+(?P<body>.+))?$").unwrap();
    let CASE_PATTERN: Regex = Regex::new(r"^(?P<keyword>case)\s+(?P<value>.+?)(?:\s+in)?$").unwrap();
    let CASE_ITEM_PATTERN: Regex = Regex::new(r"^(?P<pattern>[^()]+)\)\s*(?P<body>.*?)\s*(?P<terminator>;;&|;;|;&)?$").unwrap();
    let TERMINATOR_PATTERN: Regex = Regex::new(r"^(?P<terminator>;;&|;;|;&)$").unwrap();
    if IF_PATTERN.is_match(string) {
        Some(IF_PATTERN.captures(string).unwrap())
    } else if ELSE_PATTERN.is_match(string) {
        Some(ELSE_PATTERN.captures(string).unwrap())
    } else if CASE_PATTERN.is_match(string) {
        Some(CASE_PATTERN.captures(string).unwrap())
    } else if CASE_ITEM_PATTERN.is_match(string) {
        Some(CASE_ITEM_PATTERN.captures(string).unwrap())
    } else if TERMINATOR_PATTERN.is_match(string) {
        Some(TERMINATOR_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}
