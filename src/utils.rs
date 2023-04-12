use std::iter::Peekable;

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
