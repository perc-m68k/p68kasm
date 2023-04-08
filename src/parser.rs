use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

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