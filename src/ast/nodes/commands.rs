use super::statements::Block;
use regex::{Regex, Captures};

pub struct Group {
    pub body: Block
}


pub struct Subshell {
    pub body: Block,
}

pub enum Command {
    Group(Group),
    Subshell(Subshell),
}

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let GROUP_PATTERN: Regex = Regex::new(r"^[a-z]+$").unwrap();
    let SUBSHELL_PATTERN: Regex = Regex::new(r"^[a-z]+$").unwrap();
    if GROUP_PATTERN.is_match(string) {
        Some(GROUP_PATTERN.captures(string).unwrap())
    } else if SUBSHELL_PATTERN.is_match(string) {
        Some(SUBSHELL_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}