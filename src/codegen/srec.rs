use std::{
    borrow::Cow,
    fmt::Display,
    ops::{Bound, Deref, Index, RangeBounds},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SliceRange {
    start: Bound<usize>,
    end: Bound<usize>,
}

impl SliceRange {
    pub fn new<T: RangeBounds<usize>>(value: T) -> Self {
        Self {
            start: value.start_bound().cloned(),
            end: value.end_bound().cloned(),
        }
    }

    pub const fn offset_start(&self, idx: usize) -> usize {
        match &self.start {
            Bound::Included(a) => *a + idx,
            Bound::Excluded(a) => *a + 1 + idx,
            Bound::Unbounded => idx,
        }
    }
}

impl RangeBounds<usize> for SliceRange {
    fn start_bound(&self) -> std::ops::Bound<&usize> {
        self.start.as_ref()
    }

    fn end_bound(&self) -> std::ops::Bound<&usize> {
        self.end.as_ref()
    }
}

impl<T> Index<SliceRange> for [T] {
    type Output = [T];

    fn index(&self, index: SliceRange) -> &Self::Output {
        let start = match index.start {
            Bound::Included(x) => x,
            Bound::Excluded(x) => x + 1,
            Bound::Unbounded => 0,
        };
        let end = match index.end {
            Bound::Included(x) => x + 1,
            Bound::Excluded(x) => x,
            Bound::Unbounded => self.len(),
        };
        &self[start..end]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceMaybeOwned<'a, T: Clone> {
    slice: SliceRange,
    data: Cow<'a, [T]>,
}

impl<'a, T: Clone> SliceMaybeOwned<'a, T> {
    fn split(self, idx: usize) -> (Self, Self) {
        let idx = self.slice.offset_start(idx);
        if !self.slice.contains(&idx) {
            panic!("{idx} not contained in slice")
        }
        let b = Self {
            slice: SliceRange {
                start: Bound::Included(idx),
                end: self.slice.end,
            },
            data: self.data.clone(),
        };
        let a = Self {
            slice: SliceRange {
                start: self.slice.start,
                end: Bound::Excluded(idx),
            },
            data: self.data,
        };
        (a, b)
    }
}

impl<'a, T: Clone> Deref for SliceMaybeOwned<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data.as_ref()[self.slice]
    }
}

impl<'a, T: Clone> From<&'a [T]> for SliceMaybeOwned<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            slice: SliceRange::new(..),
            data: Cow::from(value),
        }
    }
}

impl<'a, T: Clone> From<Vec<T>> for SliceMaybeOwned<'a, T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            slice: SliceRange::new(..),
            data: Cow::from(value),
        }
    }
}

pub enum Record<'a> {
    S0,
    S1(u16, SliceMaybeOwned<'a, u8>),
    S2(u32, SliceMaybeOwned<'a, u8>), // 24 byte addr
    S3(u32, SliceMaybeOwned<'a, u8>),
    S9,
}

impl<'a> Record<'a> {
    pub fn new<T: Into<SliceMaybeOwned<'a, u8>>>(addr: u32, code: T) -> Self {
        if addr <= 0xffff {
            Self::S1(addr as u16, code.into())
        } else if addr <= 0xff_ffff {
            Self::S2(addr, code.into())
        } else {
            Self::S3(addr, code.into())
        }
    }

    pub fn split_max_len(self) -> Box<dyn Iterator<Item = Record<'a>> + 'a> {
        match self {
            Record::S1(addr, code) => {
                let max_size = 80 - (2 + 2 + 4 + 2);
                if code.len() > max_size {
                    let (a, b) = code.split(max_size);
                    let addr_b = addr as u32 + a.len() as u32;
                    let b = Self::new(addr_b, b);
                    Box::new(std::iter::once(Self::S1(addr, a)).chain(b.split_max_len()))
                } else {
                    Box::new(std::iter::once(Self::S1(addr, code)))
                }
            }
            Record::S2(addr, code) => {
                let max_size = 80 - (2 + 2 + 6 + 2);
                if code.len() > max_size {
                    let (a, b) = code.split(max_size);
                    let addr_b = addr + a.len() as u32;
                    let b = Self::new(addr_b, b);
                    Box::new(std::iter::once(Self::S2(addr, a)).chain(b.split_max_len()))
                } else {
                    Box::new(std::iter::once(Self::S2(addr, code)))
                }
            }
            Record::S3(addr, code) => {
                let max_size = 80 - (2 + 2 + 8 + 2);
                if code.len() > max_size {
                    let (a, b) = code.split(max_size);
                    let addr_b = addr + a.len() as u32;
                    let b = Self::new(addr_b, b);
                    Box::new(std::iter::once(Self::S3(addr, a)).chain(b.split_max_len()))
                } else {
                    Box::new(std::iter::once(Self::S3(addr, code)))
                }
            }
            x => Box::new(std::iter::once(x)),
        }
    }
}

fn checksum(code: &[u8], addr_size: usize, addr: usize) -> u8 {
    // eprintln!("Adding {:02X}", code.len() + 1 + addr_size);
    // eprintln!("Adding {:08X}", addr);
    let res = code.len()
        + 1
        + addr_size
        + addr.to_be_bytes().into_iter().map(|x| x as usize).sum::<usize>()
        + code
            .iter()
            .map(|x| *x as usize)
            // .inspect(|x| eprintln!("Adding {x:02X}"))
            .sum::<usize>();
    // eprintln!("Result: {:X}", res);
    0xff - ((res & 0xffusize) as u8)
}

impl Display for Record<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Record::S0 => write!(f, "S004000020DB"),
            Record::S1(addr, code) => write!(
                f,
                "S1{:02X}{addr:04X}{}{:02X}",
                code.len() + 1 + 2,
                code.iter().map(|x| format!("{x:02X}")).collect::<String>(),
                checksum(code, 2, *addr as usize)
            ),
            Record::S2(addr, code) => write!(
                f,
                "S2{:02X}{:06X}{}{:02X}",
                code.len() + 1 + 3,
                *addr & 0xffffff,
                code.iter().map(|x| format!("{x:02X}")).collect::<String>(),
                checksum(code, 3, (*addr & 0xffffff) as usize)
            ),
            Record::S3(addr, code) => write!(
                f,
                "S3{:02X}{addr:08X}{}{:02X}",
                code.len() + 1 + 4,
                code.iter().map(|x| format!("{x:02X}")).collect::<String>(),
                checksum(code, 4, *addr as usize)
            ),
            Record::S9 => write!(f, "S9030000FC"),
        }
    }
}

pub struct SRec<'a>(Vec<Record<'a>>);

impl<'a> SRec<'a> {
    pub fn new(code: impl Iterator<Item = (u32, &'a [u8])>) -> Self {
        let mut res = Vec::with_capacity(code.size_hint().0 + 2);
        res.push(Record::S0);
        let mut pc = None;
        let mut last_rec: Option<(u32, Vec<u8>)> = None;
        let mut add_last_rec = |last_rec: &mut Option<(u32, Vec<u8>)>,
                                next: Option<(u32, Vec<u8>)>| {
            if let Some((addr, code)) = last_rec.take() {
                let rec = Record::new(addr, code);
                res.extend(rec.split_max_len());
            }
            *last_rec = next;
        };
        for (addr, code) in code {
            if pc.map(|x| x == addr).unwrap_or(false) {
                last_rec
                    .get_or_insert((addr, Vec::with_capacity(code.len())))
                    .1
                    .extend_from_slice(code);
            } else {
                add_last_rec(&mut last_rec, Some((addr, code.to_vec())));
            }
            pc = Some(addr + code.len() as u32)
        }
        add_last_rec(&mut last_rec, None);
        res.push(Record::S9);
        Self(res)
    }
}

impl Display for SRec<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rec in &self.0 {
            writeln!(f, "{rec}")?;
        }
        Ok(())
    }
}
