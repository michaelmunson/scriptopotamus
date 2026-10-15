
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
