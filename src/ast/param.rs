use super::types::Type;

#[derive(Debug, Clone)]
pub enum Param {
    Arg {
        name: String,
        ty: Type,
        optional: bool,
        default: Option<String>,
        repeat: bool,
    },
    Opt {
        name: String,
        flags: Vec<String>,
        ty: Type,
        optional: bool,
        default: Option<String>,
        repeat: bool,
    },
    Rest {
        name: String,
        ty: Type,
    },
    Group {
        optional: bool,
        params: Vec<Param>,
    },
    Exclusive {
        options: Vec<Param>,
    },
}

impl Param {
    pub fn is_optional(&self) -> bool {
        match self {
            Param::Arg { optional, .. } | Param::Opt { optional, .. } => *optional,
            Param::Rest { .. } => true,
            Param::Group { optional, .. } => *optional,
            Param::Exclusive { options } => options.iter().any(|o| o.is_optional()),
        }
    }
}
