use std::{borrow::Cow, collections::HashMap, fs::File, io::Write, path::{Path, PathBuf}};

use args::Config;
use clap::Parser as ArgsParser;
use codegen::{code_for_statement, statements, symbols::NonFailingMap};
use parser::{ASMParser, Rule};
use pest::{iterators::Pairs, Parser, Span};
use typed_arena::Arena;

use crate::{args::Args, codegen::srec::SRec, listing::Listing};

mod args;
mod codegen;
mod listing;
mod parser;

struct FileAndContents<'a> {
    file: Cow<'a, Path>,
    contents: String
}

fn run_passes<'a>(
    x: Pairs<'a, Rule>,
    pc_og: u32,
    file: &'a Path,
    arena: &'a Arena<FileAndContents<'a>>,
    files: &mut Vec<&FileAndContents<'a>>,
    listing: &mut Listing<'a>,
    symbols: &mut HashMap<&'a str, u32>,
    code_object: &mut Vec<(u32, Vec<u8>)>,
    create_listing: bool,
) -> u32 {
    let statements = statements(x);
    let mut pc = pc_og;
    let mut include_end_addr = Vec::new();
    for s in statements.clone() {
        if s.as_rule() == Rule::include {
            let include_str = s.into_inner().next().unwrap().as_str().trim_end();
            let mut include_path = PathBuf::from(include_str);
            if include_path.is_relative() {
                include_path = file.parent().unwrap().join(include_path)
            }
            let string = std::fs::read_to_string(&include_path).unwrap();
            let current = &*arena.alloc(FileAndContents::<'a> { file: include_path.into(), contents: string });
            let file = &current.file;
            let file_str = &current.contents;
            files.push(current);
            match ASMParser::parse(Rule::program, file_str) {
                Ok(x) => {
                    pc = run_passes(x, pc, file, arena, files, listing, symbols, code_object, create_listing);
                    include_end_addr.push(pc);
                }
                Err(e) => println!("{} {e}", file.display()),
            }
            
        } else {
            // let span = s.as_span();
            let (label, start_addr, code) =
                code_for_statement(s, pc, &NonFailingMap(&*symbols), &file.display(), true);
            pc = start_addr.unwrap_or(pc);
            if let Some(label) = label {
                let label_span = label.into_inner().next().unwrap();
                let label = label_span.as_str();
                if !symbols.contains_key(label) {
                    symbols.insert(label, pc);
                } else {
                    let line_col = label_span.line_col();
                    panic!(
                        "Symbol `{label}` already defined ({}:{}:{})",
                        file.display(),
                        line_col.0,
                        line_col.1
                    )
                }
            }
            pc += code.len() as u32;
            // listing.insert((&file, span.start_pos().line_col().0), (start_addr, code));
        }
    }
    // println!("{symbols:#?}");
    let mut include_end_addr = include_end_addr.into_iter();
    pc = pc_og;
    for s in statements {
        // println!("{pc:08X} RULE {:?}", s.as_rule());
        if s.as_rule() == Rule::include {
            // println!("PC BEFORE: {pc:X}");
            pc = include_end_addr.next().unwrap_or(pc);
            // println!("PC: {pc:X}");
        } else {
            let span = s.as_span();
            let (_, start_addr, code) = code_for_statement(s, pc, &*symbols, &file.display(), false);
            pc = start_addr.unwrap_or(pc);
            let code_len = code.len();
            let idx = code_object.len();
            println!("{pc:08X} {code:02X?}");
            code_object.push((pc, code));
            if create_listing {
                for line in span.lines_span().filter(|a| !a.as_str().trim_end().is_empty()) {
                    let line_end = if line.start_pos().line_col().0 == span.end_pos().line_col().0 {
                        span.end_pos()
                    }else{
                        line.end_pos()
                    };
                    span.
                    println!("{:03} {line:?} ", line.start_pos().line_col().0);
                }
                listing.add(file, span.start_pos().line_col().0, idx);
            }
            pc += code_len as u32;
        }
    }
    // println!("{} {pc_og:X}->{pc:X}", file.display());
    pc
}

fn run(conf: &Config) {
    let arena = Arena::new();
    let initial = arena.alloc(FileAndContents { file: Cow::from(&conf.input_file),
        contents: std::fs::read_to_string(&conf.input_file).unwrap(), });
    
    let file = &initial.file;
    let file_str = &initial.contents;
    let mut files = vec![&*initial];
    let successful_parse = ASMParser::parse(Rule::program, file_str);

    let mut symbols = HashMap::<&str, u32>::new();
    let mut code_object = Vec::<(u32, Vec<u8>)>::new();
    let mut listing = Listing::new();
    let create_listing = conf.listing.is_some();
    match successful_parse {
        Ok(x) => {
            // let statements = statements(x);
            // let mut pc = 0;
            // for s in statements.clone() {
            //     // let span = s.as_span();
            //     let (label, start_addr, code) =
            //         code_for_statement(s, pc, &NonFailingMap(&symbols), &file.display());
            //     pc = start_addr.unwrap_or(pc);
            //     if let Some(label) = label {
            //         let label_span = label.into_inner().next().unwrap();
            //         let label = label_span.as_str();
            //         if !symbols.contains_key(label) {
            //             symbols.insert(label, pc);
            //         } else {
            //             let line_col = label_span.line_col();
            //             panic!(
            //                 "Symbol `{label}` already defined ({}:{}:{})",
            //                 file.display(),
            //                 line_col.0,
            //                 line_col.1
            //             )
            //         }
            //     }
            //     pc += code.len() as u32;
            //     // listing.insert((&file, span.start_pos().line_col().0), (start_addr, code));
            // }
            // // println!("{symbols:#?}");
            // pc = 0;
            // for s in statements {
            //     let span = s.as_span();
            //     let (_, start_addr, code) = code_for_statement(s, pc, &symbols, &file.display());
            //     pc = start_addr.unwrap_or(pc);
            //     let code_len = code.len();
            //     let idx = code_object.len();
            //     code_object.push((pc, code));
            //     if create_listing {
            //         listing.add(file, span.start_pos().line_col().0, idx);
            //     }
            //     pc += code_len as u32;
            // }
            run_passes(
                x,
                0,
                file,
                &arena,
                &mut files,
                &mut listing,
                &mut symbols,
                &mut code_object,
                create_listing,
            );
        }
        Err(e) => println!("{} {e}", file.display()),
    }
    if let Some(listing_path) = &conf.listing {
        let mut f = File::create(listing_path).unwrap();
        for FileAndContents { file, contents } in &files {
            f.write_all(
                listing
                    .printable(&code_object, file, contents)
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        }
        f.flush().unwrap();
    }
    std::fs::write(
        &conf.out,
        SRec::new(code_object.iter().map(|(a, b)| (*a, b.as_slice()))).to_string(),
    )
    .unwrap();
}

fn main() {
    let conf = Args::parse().config();
    run(&conf);
    println!("Code generated");
}
