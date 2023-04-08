use parser::{ASMParser, Rule};
use pest::Parser;

mod parser;

fn main() {
    let successful_parse = ASMParser::parse(Rule::program, include_str!("../example_asm/bib_aux.s"));
    match successful_parse {
        Ok(x) => println!("{x:#?}"),
        Err(e) => println!("{e}")
    }
}
