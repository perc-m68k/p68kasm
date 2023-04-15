use std::{borrow::Cow, fmt::Display};

use pest::{error::InputLocation, Position, Span};
use thiserror::Error;

use crate::{file::FileRef, parser::Rule, utils::PrintIteratorSep};

#[derive(Debug, Error)]
pub struct SymbolError<'a> {
    symbol_loc: Span<'a>,
    file: FileRef<'a>,
}

impl<'a> SymbolError<'a> {
    pub const fn new(symbol_loc: Span<'a>, file: FileRef<'a>) -> Self {
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
            self.file.path.display(),
        )
    }
}

pub type SymbolResult<'a, T> = Result<T, Vec<SymbolError<'a>>>;

pub fn map_op_bin<'a, T, U, F: FnOnce(T, T) -> U>(
    lhs: SymbolResult<'a, T>,
    rhs: SymbolResult<'a, T>,
    f: F,
) -> SymbolResult<'a, U> {
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
    #[error("Undefined symbols: {0:?}")]
    UndefinedSymbols(Vec<SymbolError<'a>>),
    #[error("{err}")]
    Parse {
        err: Box<pest::error::Error<Rule>>,
        file: FileRef<'a>,
    },
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

impl<'a> CodeError<'a> {
    pub fn as_display<'b, RD: RuleDisplay<'b, Rule>>(
        &'b self,
        rd: &'b RD,
    ) -> Box<dyn Iterator<Item = SpanErrorDisplay<'a>> + 'b> {
        match self {
            Self::UndefinedSymbols(v) => Box::new(v.iter().map(SpanErrorDisplay::<'a>::from)),
            Self::Parse { err, file } => Box::new(std::iter::once(SpanErrorDisplay::<'a>::from((
                err.as_ref(),
                *file,
                rd,
            )))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ErrorLocation {
    Single(usize),
    Span(usize, usize),
}

impl From<InputLocation> for ErrorLocation {
    fn from(value: InputLocation) -> Self {
        match value {
            InputLocation::Pos(x) => Self::Single(x),
            InputLocation::Span((a, b)) => Self::Span(a, b),
        }
    }
}

impl ErrorLocation {
    fn split(self, input: &str) -> (Position, Option<Position>) {
        match self {
            Self::Single(x) => (Position::new(input, x).unwrap(), None),
            Self::Span(a, b) => (
                Position::new(input, a).unwrap(),
                Some(Position::new(input, b).unwrap()),
            ),
        }
    }
}

pub enum ErrorKind {
    Error,
    Warning,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "[ERR]"),
            Self::Warning => write!(f, "[WARN]"),
        }
    }
}

pub struct SpanErrorDisplay<'a> {
    position: ErrorLocation,
    file: FileRef<'a>,
    kind: ErrorKind,
    message: String,
    note: Option<Cow<'static, str>>,
}

impl<'a> From<&SymbolError<'a>> for SpanErrorDisplay<'a> {
    fn from(value: &SymbolError<'a>) -> Self {
        Self {
            position: ErrorLocation::Span(value.symbol_loc.start(), value.symbol_loc.end()),
            file: value.file,
            kind: ErrorKind::Error,
            message: format!("symbol `{}` is undefined", value.symbol_loc.as_str()),
            note: Some("Maybe it is on an expression for ORG, in which case the symbol has to be defined before this line".into())
        }
    }
}

pub trait RuleDisplay<'a, R> {
    type Displayable: Display + 'a;

    fn rule_as_display(&self, r: &'a R) -> Self::Displayable;
}

pub struct NoChange;

impl<'a, R: Display + 'a> RuleDisplay<'a, R> for NoChange {
    type Displayable = &'a R;

    fn rule_as_display(&self, r: &'a R) -> Self::Displayable {
        r
    }
}

impl<'a, R: 'a, U: Display + 'a, F: Fn(&'a R) -> U> RuleDisplay<'a, R> for F {
    type Displayable = U;

    fn rule_as_display(&self, r: &'a R) -> Self::Displayable {
        self(r)
    }
}

impl<'a, 'b, R, RD: RuleDisplay<'b, R>> From<(&'b pest::error::Error<R>, FileRef<'a>, &'b RD)>
    for SpanErrorDisplay<'a>
{
    fn from((error, file, rd): (&'b pest::error::Error<R>, FileRef<'a>, &'b RD)) -> Self {
        Self {
            position: error.location.clone().into(),
            file,
            kind: ErrorKind::Error,
            message: match &error.variant {
                pest::error::ErrorVariant::ParsingError {
                    positives,
                    negatives,
                } => match (positives.is_empty(), negatives.is_empty()) {
                    (true, true) => "unknown parsing error".into(),
                    (true, false) => format!(
                        "unexpected {}",
                        PrintIteratorSep::new(
                            negatives.iter().map(|x| rd.rule_as_display(x)),
                            ", "
                        )
                    ),
                    (false, true) => format!(
                        "expected {}",
                        PrintIteratorSep::new(
                            positives.iter().map(|x| rd.rule_as_display(x)),
                            ", "
                        )
                    ),
                    (false, false) => format!(
                        "unexpected {}; expected: {}",
                        PrintIteratorSep::new(
                            negatives.iter().map(|x| rd.rule_as_display(x)),
                            ", "
                        ),
                        PrintIteratorSep::new(
                            positives.iter().map(|x| rd.rule_as_display(x)),
                            ", "
                        )
                    ),
                },
                pest::error::ErrorVariant::CustomError { message } => message.clone(),
            },
            note: None,
        }
    }
}

const fn in_range(
    (sl, sc): (usize, usize),
    end: Option<(usize, usize)>,
    (pl, pc): (usize, usize),
) -> bool {
    if let Some((el, ec)) = end {
        !((pl < sl || pl > el) || (sl == pl && pc < sc) || (el == pl && pc >= ec))
    } else {
        sl == pl && sc == pc
    }
}

fn multiply_if_tab(c: char, s: &'static str) -> Cow<'static, str> {
    if c == '\t' {
        format!("{0}{0}{0}{0}", s).into()
    } else {
        s.into()
    }
}

impl<'a> Display for SpanErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const MARGIN: usize = 1;
        const TAB: &str = "    ";

        let (start, end) = self.position.split(self.file.str);
        let (sline, scol) = start.line_col();
        let end_lcol = end.as_ref().map(|x| x.line_col());
        let end_line = end_lcol.map(|(l, _)| l).unwrap_or(sline);
        let linen = 2 * MARGIN + end_line - sline + 1;
        writeln!(
            f,
            "{} {} -> {}:{}:{}",
            self.kind,
            self.message,
            self.file.path.display(),
            sline,
            scol
        )?;
        for (n, line) in self
            .file
            .str
            .lines()
            .enumerate()
            .skip(sline - MARGIN - 1)
            .take(linen)
            .map(|(i, l)| (i + 1, l))
        {
            writeln!(f, "{n:>5} | {}", line.replace('\t', TAB))?;
            if (sline..=end_line).contains(&n) {
                writeln!(
                    f,
                    "{:>5} | {}",
                    "",
                    line.chars()
                        .enumerate()
                        .map(|(i, c)| (i + 1, c))
                        .map(|(pc, c)| multiply_if_tab(
                            c,
                            if in_range((sline, scol), end_lcol, (n, pc)) {
                                "^"
                            } else {
                                " "
                            }
                        ))
                        .collect::<String>()
                )?;
            }
        }
        if let Some(note) = self.note.as_ref() {
            writeln!(f, "note: {note}")?;
        }
        Ok(())
    }
}
