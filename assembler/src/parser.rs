use once_cell::sync::Lazy;
use pest::{
    iterators::Pairs,
    pratt_parser::{Assoc, Op, PrattParser},
};
use pest_derive::Parser;

use crate::{
    codegen::symbols::SymbolMap,
    error::{map_op_bin, SymbolError},
    file::FileRef,
};

#[derive(Parser)]
#[grammar = "asm2.pest"]
pub struct ASMParser;

static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::add_op, Assoc::Left) | Op::infix(Rule::subtract, Assoc::Left))
        .op(Op::infix(Rule::multiply, Assoc::Left)
            | Op::infix(Rule::divide, Assoc::Left)
            | Op::infix(Rule::modulo, Assoc::Left))
        .op(Op::infix(Rule::and_op, Assoc::Left) | Op::infix(Rule::or_op, Assoc::Left))
        .op(Op::infix(Rule::lshift, Assoc::Right) | Op::infix(Rule::rshift, Assoc::Right))
        .op(Op::prefix(Rule::neg_op) | Op::prefix(Rule::not_op))
});

pub fn parse_expression<'b, M: SymbolMap>(
    pairs: Pairs<'b, Rule>,
    symbols: &M,
    current_file: FileRef<'b>,
) -> Result<i32, Vec<SymbolError<'b>>> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::expression => parse_expression(primary.into_inner(), symbols, current_file),
            Rule::atom => {
                let inner = primary.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::expression => parse_expression(inner.into_inner(), symbols, current_file),
                    Rule::dec_number => Ok(inner
                        .as_str()
                        .parse()
                        .or_else(|_| inner.as_str().parse::<u32>().map(|x| x as i32))
                        .unwrap()),
                    Rule::hex_number => Ok(i32::from_str_radix(&inner.as_str()[1..], 16)
                        .or_else(|_| {
                            u32::from_str_radix(&inner.as_str()[1..], 16).map(|x| x as i32)
                        })
                        .unwrap()),
                    Rule::oct_number => Ok(i32::from_str_radix(&inner.as_str()[1..], 8)
                        .or_else(|_| u32::from_str_radix(&inner.as_str()[1..], 8).map(|x| x as i32))
                        .unwrap()),
                    Rule::bin_number => Ok(i32::from_str_radix(&inner.as_str()[1..], 2)
                        .or_else(|_| u32::from_str_radix(&inner.as_str()[1..], 2).map(|x| x as i32))
                        .unwrap()),
                    Rule::symbol => {
                        symbols.get(inner.as_str()).map_or_else(
                            || {
                                // let span_start = inner.as_span().start_pos().line_col();
                                Err(vec![SymbolError::new(inner.as_span(), current_file)])
                                // panic!("Symbol `{}` undefined ({}:{}:{}) (Maybe it is on an expression for ORG, in which case the symbol has to be defined before this line)", inner.as_str(), current_file, span_start.0, span_start.1)
                            },
                            |val| Ok(val as i32),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        })
        .map_prefix(|op, data| match op.as_rule() {
            Rule::neg_op => data.map(|x| -x),
            Rule::not_op => data.map(|x| !x),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add_op => map_op_bin(lhs, rhs, |a, b| a + b),
            Rule::subtract => map_op_bin(lhs, rhs, |a, b| a - b),
            Rule::multiply => map_op_bin(lhs, rhs, |a, b| a * b),
            Rule::divide => map_op_bin(lhs, rhs, |a, b| a / b),
            Rule::modulo => map_op_bin(lhs, rhs, |a, b| a % b),
            Rule::and_op => map_op_bin(lhs, rhs, |a, b| a & b),
            Rule::or_op => map_op_bin(lhs, rhs, |a, b| a | b),
            Rule::lshift => map_op_bin(lhs, rhs, |a, b| a << b),
            Rule::rshift => map_op_bin(lhs, rhs, |a, b| a >> b),
            _ => unreachable!(),
        })
        .parse(pairs)
}

#[cfg(test)]
mod test {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_example() {
        if let Err(e) = ASMParser::parse(Rule::program, include_str!("../example_asm/example.s")) {
            panic!("{e}")
        }
    }

    // #[test]
    // fn test_matrix_multiply() {
    // 	if let Err(e) = ASMParser::parse(Rule::program, include_str!("../example_asm/MatrixMultiply.s")) {
    // 		panic!("{e}")
    // 	}
    // }

    // #[test]
    // fn test_timer() {
    // 	if let Err(e) = ASMParser::parse(Rule::program, include_str!("../example_asm/timer.s")) {
    // 		panic!("{e}")
    // 	}
    // }
}
