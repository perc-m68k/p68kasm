use std::{borrow::Cow, fmt::Display, path::Path};

#[derive(Debug, Clone)]
pub struct File<'a>(Cow<'a, Path>);

impl<'a, T: Into<Cow<'a, Path>>> From<T> for File<'a> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Display for File<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
