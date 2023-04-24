use std::{fmt::Display, iter::Peekable};

pub struct IsLast<I: Iterator> {
    iter: Peekable<I>,
}

impl<I: Iterator> Iterator for IsLast<I> {
    type Item = (I::Item, bool);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| (x, self.iter.peek().is_none()))
    }
}

pub trait IteratorExt: Iterator {
    fn with_last(self) -> IsLast<Self>
    where
        Self: Sized,
    {
        IsLast {
            iter: self.peekable(),
        }
    }
}

impl<I: Iterator> IteratorExt for I {}

pub struct PrintIteratorSep<S: Display, T: Display, I: Iterator<Item = T> + Clone> {
    iter: I,
    sep: S,
}

impl<S: Display, T: Display, I: Iterator<Item = T> + Clone> PrintIteratorSep<S, T, I> {
    pub const fn new(iter: I, sep: S) -> Self {
        Self { iter, sep }
    }
}

impl<S: Display, T: Display, I: Iterator<Item = T> + Clone> Display for PrintIteratorSep<S, T, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (item, last) in self.iter.clone().with_last() {
            write!(f, "{item}")?;
            if !last {
                write!(f, "{}", self.sep)?;
            }
        }
        Ok(())
    }
}
