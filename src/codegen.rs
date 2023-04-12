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
    dry_run: bool,
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
                code_for_instr(instr, pc, symbols, current_file, dry_run),
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
        x => unreachable!("{x:?}"),
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
        Rule::address_indirect => (
            0b010,
            p.into_inner().next().unwrap().as_str().parse().unwrap(),
            vec![],
        ),
        Rule::address_indirect_postinc => (
            0b011,
            p.into_inner().next().unwrap().as_str().parse().unwrap(),
            vec![],
        ),
        Rule::address_indirect_predecr => (
            0b100,
            p.into_inner().next().unwrap().as_str().parse().unwrap(),
            vec![],
        ),
        Rule::address_indirect_disp => {
            // dbg!(&p);
            let mut inner = p.into_inner();
            let d16 = inner.next().unwrap();
            let disp = parse_expression(d16.into_inner(), symbols, current_file) as u16;
            let reg_no = inner.next().unwrap().as_str().parse::<u8>().unwrap();
            (0b101, reg_no & 0b111, disp.to_be_bytes().to_vec())
        }
        Rule::absolute_short => {
            let value = parse_expression(p.into_inner(), symbols, current_file);
            (
                0b111,
                0b001,
                ((value & 0xFFFF) as u16).to_be_bytes().to_vec(),
            )
        }
        Rule::absolute_long => {
            let value = parse_expression(p.into_inner(), symbols, current_file);
            (0b111, 0b001, value.to_be_bytes().to_vec())
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SmallSize {
    B,
    W,
}

fn small_size_to_enum(p: &Pair<Rule>) -> SmallSize {
    match p.as_span().as_str().to_uppercase().as_str() {
        ".B" => SmallSize::B,
        ".W" => SmallSize::W,
        _ => unreachable!(),
    }
}

fn code_for_instr<M: SymbolMap, F: Display>(
    p: Pair<Rule>,
    pc: u32,
    symbols: &M,
    current_file: &F,
    dry_run: bool,
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
            let mut v = ((match size {
                IntSize::B => 0b01,
                IntSize::W => 0b11,
                IntSize::L => 0b10,
            } << 12)
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
        Rule::MOVEA => {
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
            let reg_no: u16 = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            
            let mut v = ((match size {
                IntSize::B => 0b01,
                IntSize::W => 0b11,
                IntSize::L => 0b10,
            } << 12)
                | ((reg_no) << 9)
                | (0b001 << 6)
                | ((src_mode as u16) << 3)
                | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            v.extend_from_slice(&src_extra);
            v
        },
        Rule::MOVEM => todo!(),
        Rule::PEA => {
            let size = IntSize::L;
            let mut inner = p.into_inner();
            let (src_mode, src_reg, src_extra) =
                get_mode_reg_extra_for_ea(inner.next().unwrap(), size, symbols, current_file);
            #[allow(clippy::unusual_byte_groupings)]
            let mut res = (0b0100100001_000_000u16 | (src_mode as u16) << 3 | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            res.extend_from_slice(&src_extra);
            res
        }
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
        Rule::ADD => todo!(),
        Rule::ADDA => {
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
            let reg_no: u8 = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            let opmode = if size == IntSize::L { 0b111 } else { 0b011 };
            // println!("SUBA.{size:?} [{src_mode:03b} {src_reg:03b} {src_extra:02X?}], A{reg_no}");
            #[allow(clippy::unusual_byte_groupings)]
            let mut opcode = (0b1101_000_000_000_000_u16
                | ((reg_no as u16) << 9)
                | (opmode << 6)
                | ((src_mode as u16) << 3)
                | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            opcode.extend_from_slice(&src_extra);
            opcode
        },
        Rule::ADDI => todo!(),
        Rule::CLR => {
            let mut inner = p.into_inner();
            let size = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .map(|p| int_size_to_enum(&p))
                .unwrap_or_default();
            let (dst_mode, dst_reg, dst_extra) =
                get_mode_reg_extra_for_ea(inner.next().unwrap(), size, symbols, current_file);
            let size = match size {
                IntSize::B => 0b00,
                IntSize::W => 0b01,
                IntSize::L => 0b10,
            };
            #[allow(clippy::unusual_byte_groupings)]
            let mut res =
                (0b01000010_00_000000u16 | size << 6 | ((dst_mode as u16) << 3) | (dst_reg as u16))
                    .to_be_bytes()
                    .to_vec();
            res.extend_from_slice(&dst_extra);
            res
        }
        Rule::CMP => {
            // println!("{p:#?}");
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
            let reg_no: u16 = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            let mut res = (0b1011_0000_0000_0000u16
                | (reg_no << 9)
                | ((src_mode as u16) << 3)
                | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            res.extend_from_slice(&src_extra);
            res
        }
        Rule::CMPA => todo!(),
        Rule::CMPI => {
            let mut inner = p.into_inner();
            let size = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .map(|p| int_size_to_enum(&p))
                .unwrap_or_default();
            let value = parse_expression(inner.next().unwrap().into_inner(), symbols, current_file);
            let (mode, reg, extra) = get_mode_reg_extra_for_ea(inner.next().unwrap(), size, symbols, current_file);
            let bits_size = match size {
                IntSize::B => 0b00,
                IntSize::W => 0b01,
                IntSize::L => 0b10
            };
            let opcode = 0b0000110000000000u16 | (bits_size << 6) | ((mode as u16) << 3) | (reg as u16);
            let mut res = opcode.to_be_bytes().to_vec();
            match size {
                IntSize::B => res.extend_from_slice(&((value as u16) & 0xFF).to_be_bytes()),
                IntSize::W => res.extend_from_slice(&(value as u16).to_be_bytes()),
                IntSize::L => res.extend_from_slice(&(value as u32).to_be_bytes()),
            }
            res.extend_from_slice(&extra);
            res
        },
        Rule::SUB => todo!(),
        Rule::SUBA => {
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
            let reg_no: u8 = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            let opmode = if size == IntSize::L { 0b111 } else { 0b011 };
            // println!("SUBA.{size:?} [{src_mode:03b} {src_reg:03b} {src_extra:02X?}], A{reg_no}");
            #[allow(clippy::unusual_byte_groupings)]
            let mut opcode = (0b1001_000_000_000_000_u16
                | ((reg_no as u16) << 9)
                | (opmode << 6)
                | ((src_mode as u16) << 3)
                | (src_reg as u16))
                .to_be_bytes()
                .to_vec();
            opcode.extend_from_slice(&src_extra);
            opcode
        }
        Rule::SUBI => todo!(),
        Rule::Bcc => {
            if dry_run {
                vec![0, 0, 0, 0]
            } else {
                // println!("{p:#?}");
                let mut inner = p.into_inner();
                let (cc, _size) = {
                    let mut mnemonic = inner.next().unwrap().into_inner();
                    let cc = mnemonic.next().unwrap().as_str();
                    (
                        cc,
                        mnemonic
                            .next()
                            .map(|p| int_size_to_enum(&p))
                            .unwrap_or_default(),
                    )
                };
                // HI High 0010 C Λ Z
                // LS Low or Same 0011 C V Z
                // CC(HI) Carry Clear 0100 C
                // CS(LO) Carry Set 0101 C
                // NE Not Equal 0110 Z
                // EQ Equal 0111 Z
                // VC Overflow Clear 1000 V
                // VS Overflow Set 1001 V
                // PL Plus 1010 N
                // MI Minus 1011 N
                // GE Greater or Equal 1100 N Λ V V N Λ V
                // LT Less Than 1101 N Λ V V N Λ V
                // GT Greater Than 1110 N Λ V Λ Z V N Λ V Λ Z
                // LE Less or Equal 1111 Z V N Λ V V N Λ Vç
                let cc = match cc.to_uppercase().as_str() {
                    "HI" => 0b0010,
                    "LS" => 0b0011,
                    "CC" => 0b0100,
                    "CS" => 0b0101,
                    "NE" => 0b0110,
                    "EQ" => 0b0111,
                    "VC" => 0b1000,
                    "VS" => 0b1001,
                    "PL" => 0b1010,
                    "MI" => 0b1011,
                    "GE" => 0b1100,
                    "LT" => 0b1101,
                    "GT" => 0b1110,
                    "LE" => 0b1111,
                    x => unreachable!("Unexpected cc `{x}`"),
                };
                let symbol = inner.next().unwrap();
                let symbol_value = symbols.get(symbol.as_str()).expect("label to jump to") as i32;
                let disp = (((symbol_value - ((pc + 2) as i32)) as u32) as u16) as i16;
                let mut res = (0b0110_0000_0000_0000u16 | cc << 8).to_be_bytes().to_vec();
                res.extend_from_slice(&disp.to_be_bytes());
                res
            }
        }
        Rule::BRA => todo!(),
        Rule::BSR => {
            if dry_run {
                vec![0, 0, 0, 0]
            } else {
                // let span = p.as_span();
                let mut inner = p.into_inner();
                let _ = inner
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .map(|p| small_size_to_enum(&p))
                    .unwrap_or(SmallSize::W);
                // println!("BSR {size:?}");
                let symbol = inner.next().unwrap();
                let symbol_value = symbols.get(symbol.as_str()).expect("label to jump to") as i32;
                // println!("jump to {symbol_value:08X} from {pc:08X}");
                let disp = (((symbol_value - ((pc + 2) as i32)) as u32) as u16) as i16;
                // if size == SmallSize::B && (disp < i8::MIN as i16) && (disp > i8::MAX as i16) {
                //     let pos = span.start_pos().line_col();
                //     println!("[WARN] Cant do a byte bsr, label is too far away, using a word jump [{current_file}:{}:{}]", pos.0, pos.1);
                //     disp -= 2;
                // }else if size == SmallSize::W {
                //     disp -= 2;
                // }
                // println!("DISP {disp}");
                let mut opcode = 0b01100001_0000_0000u16.to_be_bytes().to_vec();
                opcode.extend_from_slice(&disp.to_be_bytes());
                // println!("OPCODE {opcode:02X?}");
                opcode
            }
        }
        Rule::JMP => {
            let mut inner = p.into_inner();
            // let size = inner
            //     .next()
            //     .unwrap()
            //     .into_inner()
            //     .next()
            //     .map(|p| int_size_to_enum(&p))
            //     .unwrap_or_default();
            let (src_mode, src_reg, src_extra) =
                get_mode_reg_extra_for_ea(inner.next().unwrap(), IntSize::L, symbols, current_file);
            // let reg_no: u8 = inner.next().unwrap().into_inner().next().unwrap().as_str().parse().unwrap();
            // println!("JMP [{src_mode:03b} {src_reg:03b} {src_extra:02X?}]");
            let mut bytes = (0b0100111011000000 | ((src_mode as u16) << 3) | (src_reg as u16)).to_be_bytes().to_vec();
            bytes.extend_from_slice(&src_extra);
            bytes
        }
        Rule::JSR => todo!(),
        Rule::NOP => 0b0100111001110001u16.to_be_bytes().to_vec(),
        Rule::RTS => 0b0100111001110101u16.to_be_bytes().to_vec(),
        Rule::ANDI_to_SR => {
            let value = parse_expression(
                p.into_inner().next().unwrap().into_inner(),
                symbols,
                current_file,
            ) as u16;
            vec![
                0b00000010,
                0b01111100,
                (value >> 8) as u8,
                (value & 0xff) as u8,
            ]
        }
        Rule::MOVE_to_SR => {
            let (src_mode, src_reg, src_extra) = get_mode_reg_extra_for_ea(p.into_inner().next().unwrap(), IntSize::W, symbols, current_file);
            let mut res = (0b0100011011000000u16 | ((src_mode as u16)<< 3) | (src_reg as u16)).to_be_bytes().to_vec();
            res.extend_from_slice(&src_extra);
            res
        }
        Rule::MOVE_to_USP => {
            let reg_no: u16 = p.into_inner().next().unwrap().into_inner().next().unwrap().as_str().parse().unwrap();
            (0b0100111001100000 | reg_no).to_be_bytes().to_vec()
        }
        Rule::MOVE_from_USP => {
            let reg_no: u16 = p.into_inner().next().unwrap().into_inner().next().unwrap().as_str().parse().unwrap();
            (0b0100111001101000 | reg_no).to_be_bytes().to_vec()
        }
        Rule::RTE => 0b0100111001110011u16.to_be_bytes().to_vec(),
        Rule::BKPT => {
            let mut inner = p.into_inner();
            let vector = inner
                .next()
                .map(|x| parse_expression(x.into_inner(), symbols, current_file))
                .unwrap_or(0) as u32;
            (0b0100100001001000 | ((vector & 0b111) as u16))
                .to_be_bytes()
                .to_vec()
        }
        Rule::TRAP => {
            let value = parse_expression(
                p.into_inner().next().unwrap().into_inner(),
                symbols,
                current_file,
            ) as u32;
            (0b010011100100_0000 | ((value & 0b1111) as u16))
                .to_be_bytes()
                .to_vec()
        }
        x => unreachable!("{x:?}"),
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
