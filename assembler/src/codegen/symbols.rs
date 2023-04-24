use std::collections::HashMap;

pub trait SymbolMap {
    fn get(&self, s: &str) -> Option<u32>;
    type Failing: SymbolMap;
    fn get_failing(&self) -> &Self::Failing;
}

// pub struct AllZeroMap;

// impl SymbolMap for AllZeroMap {
//     fn get(&self, _: &str) -> Option<u32> {
//         Some(0)
//     }
// }

pub struct NonFailingMap<M>(pub M);

// impl<M: SymbolMap> SymbolMap for NonFailingMap<M> {
//     fn get(&self, s: &str) -> Option<u32> {
//         Some(self.0.get(s).unwrap_or(0))
//     }
// }

impl<'a, M: SymbolMap> SymbolMap for NonFailingMap<&'a M> {
    fn get(&self, s: &str) -> Option<u32> {
        Some(self.0.get(s).unwrap_or(0))
    }

    type Failing = M;

    fn get_failing(&self) -> &Self::Failing {
        self.0
    }
}

// impl<'a, M: SymbolMap> SymbolMap for &'a M {
//     fn get(&self, s: &str) -> Option<u32> {
//         self.get(s)
//     }
// }

impl<'a> SymbolMap for HashMap<&'a str, u32> {
    fn get(&self, s: &str) -> Option<u32> {
        self.get(s).copied()
    }

    type Failing = Self;

    fn get_failing(&self) -> &Self::Failing {
        self
    }
}

impl SymbolMap for HashMap<String, u32> {
    fn get(&self, s: &str) -> Option<u32> {
        self.get(s).copied()
    }

    type Failing = Self;

    fn get_failing(&self) -> &Self::Failing {
        self
    }
}
