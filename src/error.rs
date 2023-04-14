use std::fmt::Display;

use pest::Span;
use thiserror::Error;

use crate::parser::Rule;

#[derive(Debug, Error)]
pub struct SymbolError<'a, F = String>
where
    F: Display,
{
    symbol_loc: Span<'a>,
    file: F,
}

impl<'a, F> SymbolError<'a, F>
where
    F: Display,
{
    pub const fn new(symbol_loc: Span<'a>, file: F) -> Self {
        Self { symbol_loc, file }
    }
}

impl Display for SymbolError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, col) = self.symbol_loc.start_pos().line_col();
        write!(
            f,
            "symbol undefined [{}] @ {}:{line}:{col}",
            self.symbol_loc.as_str(),
            self.file
        )
    }
}

pub type SymbolResult<'a, T, D> = Result<T, Vec<SymbolError<'a, D>>>;

pub fn map_op_bin<'a, T, U, D: Display, F: FnOnce(T, T) -> U>(
    lhs: SymbolResult<'a, T, D>,
    rhs: SymbolResult<'a, T, D>,
    f: F,
) -> SymbolResult<'a, U, D> {
    match (lhs, rhs) {
        (Ok(lhs), Ok(rhs)) => Ok(f(lhs, rhs)),
        (Err(mut lhs), Err(rhs)) => {
            lhs.extend(rhs);
            Err(lhs)
        }
        (Err(x), Ok(_)) | (Ok(_), Err(x)) => Err(x),
    }
}

#[derive(Debug, Error)]
pub enum CodeError<'a> {
    UndefinedSymbols(Vec<SymbolError<'a>>),
    Parse(#[from] Box<pest::error::Error<Rule>>)
}

impl<'a> Display for CodeError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> From<Vec<SymbolError<'a>>> for CodeError<'a> {
    fn from(value: Vec<SymbolError<'a>>) -> Self {
        Self::UndefinedSymbols(value)
    }
}

impl<'a> From<SymbolError<'a>> for CodeError<'a> {
    fn from(value: SymbolError<'a>) -> Self {
        vec![value].into()
    }
}
