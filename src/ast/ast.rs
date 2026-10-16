use super::nodes::statements::{Block, capture};
use std::fs::File;
use std::io::{BufReader, Read};

pub type Ast = Vec<Block>;

pub fn parse(filename: &str) {
    let mut file = File::open(filename).expect("Unable to open file");
    let mut buf_reader = BufReader::new(file);
    let mut filetext = String::new();
    buf_reader.read_to_string(&mut filetext).expect("Unable to read file");
    // Placeholder: Return empty AST until further parsing is implemented
    let captures = capture(filetext.as_str());
    if captures.is_some() {
        let captures = captures.unwrap();
        println!("Captures: {:?}", captures);
    } else {
        println!("No captures");
    }
}