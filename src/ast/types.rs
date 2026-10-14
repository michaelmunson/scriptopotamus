pub enum Type {
    Integer,
    Float,
    String,
    Boolean,
    Array(Box<Type>),
    Dict(Box<Type>, Box<Type>),
}