use std::{collections::HashMap, path::{PathBuf, Path}};

use codegen::{statements, code_for_statement, symbols::AllZeroMap};
use parser::{ASMParser, Rule};
use pest::Parser;

mod parser;
mod codegen;

type FileId = PathBuf;

fn main() {
    let file_str = include_str!("../example_asm/simple.s");
    let successful_parse = ASMParser::parse(Rule::program, file_str);
    let file = FileId::from("../example_asm/simple.s");
    let symbols = HashMap::<&str, u32>::new();
    let mut listing = HashMap::<(&Path, usize), (Option<u32>, Vec<u8>)>::new();
    match successful_parse {
        Ok(x) => {
            let statements = statements(x);
            for s in statements.clone() {
                let span = s.as_span();
                let code = code_for_statement(s, &AllZeroMap);
                listing.insert((&file, span.start_pos().line_col().0), code);
            }
        },
        Err(e) => println!("{e}")
    }
    let mut pc = 0u32;
    for (line_no, line) in file_str.lines().enumerate().map(|(i,x)| (i+1,x)) {
        let (len, code) = if let Some((start, code)) = listing.get(&(&file, line_no)) {
            if let Some(start) = start {
                pc = *start;
            }
            (code.len() as u32, Some(code))
        }else{(0, None)};
        println!("{pc:08X} {:<30} {line_no:>5}  {}", code.into_iter().flatten().scan(0u8, |i, x| {
            let old_i = *i;
            *i = (*i + 1) % 2;
            if old_i == 1 {
                Some(format!("{x:02X} "))
            }else{
                Some(format!("{x:02X}"))
            }
        }).collect::<String>(), line.trim_end());
        pc += len;
    }
}
