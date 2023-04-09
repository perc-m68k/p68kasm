use pest::iterators::{Pair, Pairs};
use crate::parser::Rule;

pub fn code_for_statement(p: &Pair<Rule>) -> Vec<u8> {
	vec![0,0,0,0]
}

pub fn listing(pairs: Pairs<Rule>) -> impl Iterator<Item = (u32, Vec<u8>, Pair<Rule>)> {
	pairs.scan(0, |state, pair| {
		let addr = *state;
		let code = code_for_statement(&pair);
		*state += code.len() as u32;
		Some((addr, code, pair))
	})
}