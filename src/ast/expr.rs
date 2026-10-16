#[derive(Debug, Clone)]
pub enum Value {
    Scalar(String),
    List(Vec<String>),
    Dict(Vec<(String, String)>),
    Math(String),
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub kind: ConditionKind,
    pub raw: String,
    pub negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionKind {
    Bool,
    Test,
    Command,
}
