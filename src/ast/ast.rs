
pub struct VariableDeclaration {
    pub name: String,
    pub kind: VariableKind,
    pub args: Vec<VariableKind>,
    pub value: Option<Expression>,
}

pub struct IntegerVariableDeclaration {
    pub name:String = "int"
    pub args: Vec<VariableKind>,
    pub value: Option<Expression>,
}

pub struct Expression {

}

pub struct ConditionalStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub alternate: Vec<ConditionalStatement>,
}

pub enum Statement {
    Conditional(ConditionalStatement),
    Expression(Expression),
}