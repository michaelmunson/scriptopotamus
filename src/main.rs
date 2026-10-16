mod utils;
mod ast;
use ast::nodes::statements::Block;
use ast::ast::parse;

fn main() {
    parse(".scrippo");
}