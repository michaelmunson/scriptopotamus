use regex::{Regex, Captures};

pub struct Literal {
    pub value: String,
    pub datatype: Datatype,
}

pub struct Pattern {
    pub value: String
}
pub enum Datatype {
    Integer,
    Float,
    String,
    Boolean,
    Const(Box<Datatype>),
    Array(Box<Datatype>),
    Dict(Box<Datatype>, Box<Datatype>),
    Pattern(Pattern),
    Literal(Box<Literal>),
}

pub fn capture<'a>(string: &'a str) -> Option<Captures<'a>> {
    let CONST_PATTERN: Regex = Regex::new(r"^const\s+(?P<const>.+)$").unwrap();
    let ARRAY_PATTERN: Regex = Regex::new(r"^(?P<array>.+)\[\]$").unwrap();
    let DICT_PATTERN: Regex = Regex::new(r"^\{\s*(?P<key>[^:]+?)\s*:\s*(?P<value>.+?)\s*\}$").unwrap();
    let PRIMITIVE_PATTERN: Regex = Regex::new(r"^(?P<primitive>int|integer|float|str|string|bool|boolean)$").unwrap();
    let LITERAL_PATTERN: Regex = Regex::new(r#"^(?P<literal>"[^"]*"|'[^']*'|-?\d+\.\d+|-?\d+|true|false)$"#).unwrap();
    let PATTERN_PATTERN: Regex = Regex::new(r"^/(?P<pattern>.*)/$").unwrap();
    if CONST_PATTERN.is_match(string) {
        Some(CONST_PATTERN.captures(string).unwrap())
    } else if ARRAY_PATTERN.is_match(string) {
        Some(ARRAY_PATTERN.captures(string).unwrap())
    } else if DICT_PATTERN.is_match(string) {
        Some(DICT_PATTERN.captures(string).unwrap())
    } else if PRIMITIVE_PATTERN.is_match(string) {
        Some(PRIMITIVE_PATTERN.captures(string).unwrap())
    } else if LITERAL_PATTERN.is_match(string) {
        Some(LITERAL_PATTERN.captures(string).unwrap())
    } else if PATTERN_PATTERN.is_match(string) {
        Some(PATTERN_PATTERN.captures(string).unwrap())
    } else {
        None
    }
}
