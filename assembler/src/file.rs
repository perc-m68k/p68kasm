// use std::{borrow::Cow, fmt::Display, path::Path};

// #[derive(Debug, Clone)]
// pub struct File<'a>(Cow<'a, Path>);

// impl<'a, T: Into<Cow<'a, Path>>> From<T> for File<'a> {
//     fn from(value: T) -> Self {
//         Self(value.into())
//     }
// }

// impl Display for File<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0.display())
//     }
// }

use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct FileRef<'a> {
    pub path: &'a Path,
    pub str: &'a str,
}

impl<'a> FileRef<'a> {
    pub const fn new(path: &'a Path, str: &'a str) -> Self {
        Self { path, str }
    }
}
