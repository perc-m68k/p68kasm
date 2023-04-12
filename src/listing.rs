use std::{collections::HashMap, fmt::Display, path::Path};

#[derive(Debug, Default, Clone)]
pub struct Listing<'a>(HashMap<(&'a Path, usize), usize>);

impl<'a> Listing<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, file: &'a Path, line: usize, code_idx: usize) {
        self.0.insert((file, line), code_idx);
    }

    pub const fn printable<'b>(
        &'b self,
        code_object: &'b [(u32, Vec<u8>)],
        file: &'b Path,
        file_str: &'b str,
    ) -> PrintableListing<'b> {
        PrintableListing {
            listing: self,
            code_object,
            file,
            file_str,
        }
    }
}

pub struct PrintableListing<'a> {
    listing: &'a Listing<'a>,
    code_object: &'a [(u32, Vec<u8>)],
    file: &'a Path,
    file_str: &'a str,
}

struct Spaced<T>(T);

impl<T: Display> Display for Spaced<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} ", self.0)
    }
}

impl Display for PrintableListing<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "{:=^130}", Spaced(self.file.display()).to_string())?;
        writeln!(f)?;
        println!("{}", self.file.display());
        let mut pc = 0u32;
        for (line_no, line) in self.file_str.lines().enumerate().map(|(i, x)| (i + 1, x)) {

            let code = if let Some(&idx) = self.listing.0.get(&(self.file, line_no)) {
                let (addr, code) = &self.code_object[idx];
                println!("{line_no:03} BEFORE {pc:08X}");
                pc = *addr;
                println!("{line_no:03}  AFTER {pc:08X}");
                Some(code)
            } else {
                println!("{line_no:03} NO CODE");
                None
            };
            let len = code.as_ref().map(|x| x.len()).unwrap_or(0);
            writeln!(
                f,
                "{pc:08X}  {:<30} {line_no:>5}  {}",
                code.into_iter()
                    .flatten()
                    .scan(0u8, |i, x| {
                        let old_i = *i;
                        *i = (*i + 1) % 2;
                        if old_i == 1 {
                            Some(format!("{x:02X} "))
                        } else {
                            Some(format!("{x:02X}"))
                        }
                    })
                    .collect::<String>(),
                line.trim_end()
            )?;
            pc += len as u32;
        }
        Ok(())
    }
}
