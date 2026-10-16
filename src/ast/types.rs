#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Str,
    Int,
    Float,
    Bool,
    List,
    Dict,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
    pub is_const: bool,
    pub literals: Option<Vec<String>>,
}

impl Type {
    pub fn plain(kind: TypeKind) -> Self {
        Self {
            kind,
            is_const: false,
            literals: None,
        }
    }

    pub fn description(&self) -> String {
        let base = match self.kind {
            TypeKind::Str => "str",
            TypeKind::Int => "int",
            TypeKind::Float => "float",
            TypeKind::Bool => "bool",
            TypeKind::List => "list",
            TypeKind::Dict => "dict",
            TypeKind::Any => "any",
        };
        let mut parts = Vec::new();
        if self.is_const {
            parts.push("const".to_string());
        }
        parts.push(base.to_string());
        if let Some(lits) = &self.literals {
            parts.push(format!("{{{}}}", lits.join(" ")));
        }
        parts.join(" ")
    }
}
