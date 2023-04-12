use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use arena::FileArena;
use args::Config;
use clap::Parser as ArgsParser;
use codegen::{code_for_statement, statements, symbols::NonFailingMap};
use parser::{ASMParser, Rule};
use pest::{iterators::Pairs, Parser};

use crate::{args::Args, codegen::srec::SRec, listing::Listing, utils::IteratorExt};

mod arena;
mod args;
mod codegen;
mod listing;
mod parser;
mod utils;

// #[derive(Debug, Clone)]
struct CurrentFile<'a> {
    pairs: Pairs<'a, Rule>,
    path: &'a Path,
    entrypoint: u32,
}
// #[derive(Debug, Clone, Copy)]
struct GlobalData<'a> {
    arena: &'a FileArena<'a>,
    listing: Listing<'a>,
    symbols: HashMap<&'a str, u32>,
    code_object: Vec<(u32, Vec<u8>)>,
}

fn run_passes<'a>(
    current_file: CurrentFile<'a>,
    global_data: &mut GlobalData<'a>,
    create_listing: bool,
) -> u32 {
    let statements = statements(current_file.pairs);
    let mut pc = current_file.entrypoint;
    let mut include_end_addr = Vec::new();
    for s in statements.clone() {
        if s.as_rule() == Rule::include {
            let include_str = s.into_inner().next().unwrap().as_str().trim_end();
            let mut include_path = PathBuf::from(include_str);
            if include_path.is_relative() {
                include_path = current_file.path.parent().unwrap().join(include_path)
            }
            // let string = std::fs::read_to_string(&include_path).unwrap();
            let (file, file_str) = global_data.arena.add(include_path).unwrap();
            // let current = &*global_data.arena.alloc(FileAndContents::<'a> { file: include_path.into(), contents: string });
            // let file = &current.file;
            // let file_str = &current.contents;
            // global_data.files.push(current);
            match ASMParser::parse(Rule::program, file_str) {
                Ok(x) => {
                    let included_file = CurrentFile {
                        pairs: x,
                        path: file,
                        entrypoint: pc,
                    };
                    pc = run_passes(included_file, global_data, create_listing);
                    include_end_addr.push(pc);
                }
                Err(e) => println!("{} {e}", file.display()),
            }
        } else {
            // let span = s.as_span();
            let (label, start_addr, code) = code_for_statement(
                s,
                pc,
                &NonFailingMap(&global_data.symbols),
                &current_file.path.display(),
                true,
            );
            pc = start_addr.unwrap_or(pc);
            if let Some(label) = label {
                let label_span = label.into_inner().next().unwrap();
                let label = label_span.as_str();
                if !global_data.symbols.contains_key(label) {
                    global_data.symbols.insert(label, pc);
                } else {
                    let line_col = label_span.line_col();
                    panic!(
                        "Symbol `{label}` already defined ({}:{}:{})",
                        current_file.path.display(),
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
    pc = current_file.entrypoint;
    for s in statements {
        // println!("{pc:08X} RULE {:?}", s.as_rule());
        if s.as_rule() == Rule::include {
            // println!("PC BEFORE: {pc:X}");
            pc = include_end_addr.next().unwrap_or(pc);
            // println!("PC: {pc:X}");
        } else {
            let span = s.as_span();
            let (_, start_addr, code) = code_for_statement(
                s,
                pc,
                &global_data.symbols,
                &current_file.path.display(),
                false,
            );
            pc = start_addr.unwrap_or(pc);
            let code_len = code.len();
            let idx = global_data.code_object.len();
            // println!("{pc:08X} {code:02X?}");
            global_data.code_object.push((pc, code));
            if create_listing {
                for (line, last) in span
                    .lines_span()
                    .map(|line| {
                        let line_start =
                            if line.start_pos().line_col().0 == span.start_pos().line_col().0 {
                                span.start_pos()
                            } else {
                                line.start_pos()
                            };
                        let line_end =
                            if line.start_pos().line_col().0 == span.end_pos().line_col().0 {
                                span.end_pos()
                            } else {
                                line.end_pos()
                            };
                        let new_span = line_start.span(&line_end);

                        new_span
                    })
                    .filter(|line| !line.as_str().trim_end().is_empty())
                    .with_last()
                {
                    // println!("{line:?}");
                    if last {
                        global_data.listing.add(
                            current_file.path,
                            line.start_pos().line_col().0,
                            idx,
                        );
                    } else {
                        global_data.listing.add_no_code(
                            current_file.path,
                            line.start_pos().line_col().0,
                            idx,
                        );
                    }
                    // println!("{line:?} {last} {:?} {:?}", line.start_pos().line_col(), line.end_pos().line_col());
                }
            }
            pc += code_len as u32;
        }
    }
    // println!("{} {pc_og:X}->{pc:X}", file.display());
    pc
}

fn run(conf: &Config) {
    let arena = FileArena::new();
    let (file, file_str) = arena.add(&conf.input_file).unwrap();

    // let file = &initial.file;
    // let file_str = &initial.contents;
    // let files = vec![&*initial];
    let successful_parse = ASMParser::parse(Rule::program, file_str);

    let symbols = HashMap::<&str, u32>::new();
    let code_object = Vec::<(u32, Vec<u8>)>::new();
    let listing = Listing::new();
    let create_listing = conf.listing.is_some();
    let mut global_data = GlobalData {
        arena: &arena,
        listing,
        symbols,
        code_object,
    };
    match successful_parse {
        Ok(x) => {
            let current_file = CurrentFile {
                pairs: x,
                entrypoint: 0,
                path: file,
            };

            run_passes(current_file, &mut global_data, create_listing);
        }
        Err(e) => println!("{} {e}", file.display()),
    }
    if let Some(listing_path) = &conf.listing {
        let mut f = File::create(listing_path).unwrap();
        for (file, contents) in global_data.arena {
            f.write_all(
                global_data
                    .listing
                    .printable(&global_data.code_object, file, contents)
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        }
        f.flush().unwrap();
    }
    std::fs::write(
        &conf.out,
        SRec::new(
            global_data
                .code_object
                .iter()
                .map(|(a, b)| (*a, b.as_slice())),
        )
        .to_string(),
    )
    .unwrap();
}

fn main() {
    let conf = Args::parse().config();
    run(&conf);
    println!("Code generated");
}
