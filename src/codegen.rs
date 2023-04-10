use std::fmt::Display;

use crate::parser::{parse_expression, Rule};
use pest::iterators::{Pair, Pairs};

use self::symbols::SymbolMap;

pub mod srec;
pub mod symbols;

pub fn code_for_statement<'a, M: SymbolMap, F: Display>(
    p: Pair<'a, Rule>,
    pc: u32,
    symbols: &M,
    current_file: &F,
) -> (Option<Pair<'a, Rule>>, Option<u32>, Vec<u8>) {
    match p.as_rule() {
        Rule::instruction => {
            let mut inner = p.into_inner();
            let mut label = inner.next();
            let instr = label
                .take()
                .and_then(|first| {
                    if first.as_rule() == Rule::sol_label {
                        label = Some(first);
                        inner.next()
                    } else {
                        Some(first)
                    }
                })
                .unwrap();
            (
                label,
                IntSize::W.aligned(pc),
                code_for_instr(instr, symbols, current_file),
            )
        }
        Rule::org => {
            let mut inner = p.into_inner();
            let mut label = inner.next();
            let expr = label
                .take()
                .and_then(|first| {
                    if first.as_rule() == Rule::sol_label {
                        label = Some(first);
                        inner.next()
                    } else {
                        Some(first)
                    }
                })
                .unwrap();
            let expr = parse_expression(expr.into_inner(), symbols.get_failing(), current_file);
            (label, Some(expr as u32), vec![])
        }
        Rule::equ => todo!(),
        Rule::define_constant => {
            let mut inner = p.into_inner();
            let mut label = inner.next();
            let size = label
                .take()
                .and_then(|first| {
                    if first.as_rule() == Rule::sol_label {
                        label = Some(first);
                        inner.next()
                    } else {
                        Some(first)
                    }
                })
                .unwrap()
                .into_inner()
                .next()
                .map(|p| int_size_to_enum(&p))
                .unwrap_or_default();
            let mut res = Vec::new();
            for x in inner {
                data_for_item(size, x, symbols, current_file, &mut res);
            }
            (label, size.aligned(pc), res)
        }
        Rule::define_storage => todo!(),
        _ => unreachable!(),
    }
}

fn data_for_item<M: SymbolMap, F: Display>(
    size: IntSize,
    pair: Pair<Rule>,
    symbols: &M,
    current_file: &F,
    data: &mut Vec<u8>,
) {
    match pair.as_rule() {
        Rule::string => todo!(),
        Rule::expression => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col();
            let value = parse_expression(pair.into_inner(), symbols, current_file) as u32;
            match size {
                IntSize::B => {
                    if value > 0xff {
                        eprintln!("expression @ {current_file}:{}:{} is bigger than expected (byte), truncating", start_pos.0, start_pos.1)
                    }
                    data.push((value & 0xff) as u8);
                }
                IntSize::W => {
                    if value > 0xffff {
                        eprintln!("expression @ {current_file}:{}:{} is bigger than expected (word), truncating", start_pos.0, start_pos.1)
                    }
                    data.push(((value & 0xff00) >> 8) as u8);
                    data.push((value & 0xff) as u8);
                }
                IntSize::L => data.extend_from_slice(&value.to_be_bytes()),
            }
        }
        _ => unreachable!(),
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntSize {
    B = 1,
    W = 2,
    L = 4,
}

impl Default for IntSize {
    fn default() -> Self {
        Self::W
    }
}

impl IntSize {
    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::B => 0b01,
            Self::W => 0b11,
            Self::L => 0b10,
        }
    }

    pub const fn aligned(&self, pc: u32) -> Option<u32> {
        let m = pc % (*self as u32);
        if m == 0 {
            None
        } else {
            Some(pc - m + (*self as u32))
        }
    }
}

fn int_size_to_enum(p: &Pair<Rule>) -> IntSize {
    match p.as_span().as_str().to_uppercase().as_str() {
        ".B" => IntSize::B,
        ".W" => IntSize::W,
        ".L" => IntSize::L,
        _ => unreachable!(),
    }
}

fn get_mode_reg_extra_for_ea<M: SymbolMap, F: Display>(
    p: Pair<Rule>,
    size: IntSize,
    symbols: &M,
    current_file: &F,
) -> (u8, u8, Vec<u8>) {
    match p.as_rule() {
        Rule::Dn => (
            0b000,
            p.into_inner().next().unwrap().as_str().parse().unwrap(),
            vec![],
        ),
        Rule::An => (
            0b001,
            p.into_inner().next().unwrap().as_str().parse().unwrap(),
            vec![],
        ),
        Rule::address_indirect => todo!(),
        Rule::address_indirect_postinc => todo!(),
        Rule::address_indirect_predecr => todo!(),
        Rule::address_indirect_disp => todo!(),
        Rule::absolute_short => todo!(),
        Rule::absolute_long => todo!(),
        Rule::immediate_data => {
            let value = parse_expression(p.into_inner(), symbols, current_file);
            (
                0b111,
                0b100,
                match size {
                    IntSize::B => vec![0x00, (value & 0xFF) as u8],
                    IntSize::W => ((value & 0xFFFF) as u16).to_be_bytes().to_vec(),
                    IntSize::L => value.to_be_bytes().to_vec(),
                },
            )
        }
        _ => unreachable!(),
    }
}

fn code_for_instr<M: SymbolMap, F: Display>(
    p: Pair<Rule>,
    symbols: &M,
    current_file: &F,
) -> Vec<u8> {
    match p.as_rule() {
        Rule::LEA => todo!(),
        Rule::LINK => {
            let mut inner = p.into_inner();
            let an = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse::<u8>()
                .unwrap();
            let data =
                parse_expression(inner.next().unwrap().into_inner(), symbols, current_file) as u16;
            let opcode = 0b0100111001010000 | (an as u16);
            let mut res = opcode.to_be_bytes().to_vec();
            res.extend_from_slice(&data.to_be_bytes());
            res
        }
        Rule::MOVE => {
            let mut inner = p.into_inner();
            let size = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .map(|p| int_size_to_enum(&p))
                .unwrap_or_default();
            let (src_mode, src_reg, src_extra) =
                get_mode_reg_extra_for_ea(inner.next().unwrap(), size, symbols, current_file);
            let (dst_mode, dst_reg, dst_extra) =
                get_mode_reg_extra_for_ea(inner.next().unwrap(), size, symbols, current_file);
            // println!("MOVE.{size:?} {src_mode:03b} {src_reg:03b} {src_extra:02x?} {dst_mode:03b} {dst_reg:03b} {dst_extra:02x?}");
            let mut v = (((size.as_u8() as u16) << 12)
                | ((dst_reg as u16) << 9)
                | ((dst_mode as u16) << 6)
                | ((src_mode as u16) << 3)
                | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            v.extend_from_slice(&src_extra);
            v.extend_from_slice(&dst_extra);
            v
        }
        Rule::MOVEA => todo!(),
        Rule::MOVEM => todo!(),
        Rule::PEA => todo!(),
        Rule::UNLK => {
            let an = p
                .into_inner()
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse::<u8>()
                .unwrap();
            (0b0100111001011000u16 | (an as u16)).to_be_bytes().to_vec()
        }

        _ => unreachable!(),
    }
}

pub fn statements(program: Pairs<Rule>) -> impl Iterator<Item = Pair<Rule>> + Clone {
    program.flat_map(|x| x.into_inner()).filter_map(|pair| {
        if pair.as_rule() == Rule::statement {
            pair.into_inner().next()
        } else {
            None
        }
    })
}
