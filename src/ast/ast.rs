use super::nodes::statements::Block;

pub type Ast = Vec<Block>;

pub fn parse(source: &str) -> Ast {
    let mut ast = Ast::new();
    let mut current_block = Block::new();
    for line in source.lines() {
        let tokens = line.split_whitespace().collect::<Vec<&str>>();
    }
    ast
}