use std::{collections::HashMap, path::{PathBuf, Path}};

use codegen::{statements, code_for_statement, symbols::NonFailingMap};
use parser::{ASMParser, Rule};
use pest::Parser;

mod parser;
mod codegen;

type FileId = PathBuf;

fn main() {
    let file_str = include_str!("../example_asm/simple.s");
    let successful_parse = ASMParser::parse(Rule::program, file_str);
    let file = FileId::from("../example_asm/simple.s");
    let mut symbols = HashMap::<&str, u32>::new();
    let mut code_object = Vec::<(u32, Vec<u8>)>::new();
    let mut listing = HashMap::<(&Path, usize), usize>::new();
    let create_listing = true;
    match successful_parse {
        Ok(x) => {
            let statements = statements(x);
            let mut pc = 0;
            for s in statements.clone() {
                // let span = s.as_span();
                let (label, start_addr, code) = code_for_statement(s, &NonFailingMap(&symbols), &file.display());
                pc = start_addr.unwrap_or(pc);
                if let Some(label) = label {
                    let label_span = label.into_inner().next().unwrap();
                    let label = label_span.as_str();
                    symbols.insert(label, pc);
                }
                pc += code.len() as u32;
                // listing.insert((&file, span.start_pos().line_col().0), (start_addr, code));
            }
            // println!("{symbols:#?}");
            pc = 0;
            for s in statements {
                let span = s.as_span();
                let (label, start_addr, code) = code_for_statement(s, &symbols, &file.display());
                pc = start_addr.unwrap_or(pc);
                if let Some(label) = label {
                    let label_span = label.into_inner().next().unwrap();
                    let label = label_span.as_str();
                    symbols.insert(label, pc);
                }
                let code_len = code.len();
                let idx = code_object.len();
                code_object.push((pc, code));
                if create_listing {
                    listing.insert((&file, span.start_pos().line_col().0), idx);
                }
                pc += code_len as u32;
            }
        },
        Err(e) => println!("{e}")
    }
    if create_listing {
        let mut pc = 0u32;
        for (line_no, line) in file_str.lines().enumerate().map(|(i,x)| (i+1,x)) {
            let code = if let Some(&idx) = listing.get(&(&file, line_no)) {
                let (addr, code) = &code_object[idx];
                pc = *addr;
                Some(code)
            }else{None};
            let len = code.as_ref().map(|x| x.len()).unwrap_or(0);
            println!("{pc:08X}  {:<30} {line_no:>5}  {}", code.into_iter().flatten().scan(0u8, |i, x| {
                let old_i = *i;
                *i = (*i + 1) % 2;
                if old_i == 1 {
                    Some(format!("{x:02X} "))
                }else{
                    Some(format!("{x:02X}"))
                }
            }).collect::<String>(), line.trim_end());
            pc += len as u32;
        }
    }
    
}
