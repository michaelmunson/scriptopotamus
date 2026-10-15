use super::datatype::Datatype;
use super::expr::Expression;

pub struct Variable {
    pub name: String,
    pub datatype: Datatype,
    pub value: Option<Expression>
}

pub struct Reference {
    pub variable: Variable,
}