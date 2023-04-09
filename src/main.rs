use codegen::listing;
use parser::{ASMParser, Rule};
use pest::Parser;

mod parser;
mod codegen;

fn main() {
    let successful_parse = ASMParser::parse(Rule::program, include_str!("../example_asm/bib_aux.s"));
    match successful_parse {
        Ok(mut x) => {
            if let Some(x) = x.next() {
                for (i, (addr, code, pair)) in listing(x.into_inner()).enumerate() {
                    println!("{addr:08X}  {:<30} {:>5} {}", code.iter().map(|x| format!("{x:02X}")).collect::<String>(), i+1, pair.as_str().trim_end())
                }
            }
            
        },
        Err(e) => println!("{e}")
    }
}
