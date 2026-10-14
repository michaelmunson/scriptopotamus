mod utils;
mod ast;
use ast::{ast, types};

fn main() {
    println!("Hello, world!");
}

const t: types::Type = types::Type::Array(Box::new(types::Type::Integer));