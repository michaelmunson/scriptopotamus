use super::expr::{Condition, Value};
use super::param::Param;
use super::types::Type;

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration {
        name: String,
        ty: Type,
        line: usize,
    },
    Assignment {
        name: String,
        ty: Option<Type>,
        value: Value,
        line: usize,
    },
    If {
        branches: Vec<IfBranch>,
        else_body: Option<Block>,
        line: usize,
    },
    For {
        var: String,
        items: Vec<String>,
        body: Block,
        line: usize,
    },
    While {
        condition: Condition,
        body: Block,
        line: usize,
    },
    Case {
        subject: String,
        arms: Vec<CaseArm>,
        line: usize,
    },
    Echo {
        value: Value,
        line: usize,
    },
    Function {
        name: String,
        is_command: bool,
        docs: Vec<(String, String)>,
        params: Vec<Param>,
        body: Block,
        line: usize,
    },
    MacroDef {
        name: String,
        params: Vec<String>,
        body: Block,
        line: usize,
    },
    MacroCall {
        name: String,
        args: Vec<String>,
        line: usize,
    },
    Bash {
        raw: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: Condition,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct CaseArm {
    pub pattern: String,
    pub body: Block,
    pub terminator: CaseTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseTerminator {
    Break,
    Continue,
    Fallthrough,
}

impl CaseTerminator {
    pub fn as_bash(&self) -> &'static str {
        match self {
            CaseTerminator::Break => ";;",
            CaseTerminator::Continue => ";&",
            CaseTerminator::Fallthrough => ";;&",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            ";;" => Some(CaseTerminator::Break),
            ";&" => Some(CaseTerminator::Continue),
            ";;&" => Some(CaseTerminator::Fallthrough),
            _ => None,
        }
    }
}
