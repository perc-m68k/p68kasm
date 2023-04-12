use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    path::Path,
};

use typed_arena::Arena;

struct FileAndContents<'a> {
    file: Cow<'a, Path>,
    contents: String,
}

pub struct FileArena<'a> {
    arena: Arena<FileAndContents<'a>>,
    files: RefCell<Vec<&'a FileAndContents<'a>>>,
}

impl<'a> FileArena<'a> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            files: Default::default(),
        }
    }

    fn add_contents<'b: 'a, C: Into<Cow<'a, Path>>>(
        &'b self,
        path: C,
        contents: String,
    ) -> (&'a Path, &'a str) {
        let res = self.arena.alloc(FileAndContents {
            file: path.into(),
            contents,
        });
        self.files.borrow_mut().push(res);
        (&res.file, res.contents.as_str())
    }

    pub fn add<'b: 'a, C: Into<Cow<'a, Path>>>(
        &'b self,
        path: C,
    ) -> std::io::Result<(&'a Path, &'a str)> {
        let path = path.into();
        let contents = std::fs::read_to_string(&path)?;
        Ok(self.add_contents(path, contents))
    }
}

impl<'a> Default for FileArena<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a FileArena<'a> {
    type Item = <IntoIter<'a> as Iterator>::Item;

    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let r = self.files.borrow();
        IntoIter { r, idx: 0 }
    }
}

pub struct IntoIter<'a> {
    r: Ref<'a, Vec<&'a FileAndContents<'a>>>,
    idx: usize,
}

impl<'a> Iterator for IntoIter<'a> {
    type Item = (&'a Path, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self
            .r
            .get(self.idx)
            .map(|x| (x.file.as_ref(), x.contents.as_str()));
        self.idx += 1;
        res
    }
}
