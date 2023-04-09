use std::collections::HashMap;

pub trait SymbolMap {
	fn get(&self, s: &str) -> Option<u32>;
}

pub struct AllZeroMap;

impl SymbolMap for AllZeroMap {
    fn get(&self, _: &str) -> Option<u32> {
        Some(0)
    }
}

impl<'a> SymbolMap for HashMap<&'a str, u32> {
    fn get(&self, s: &str) -> Option<u32> {
        self.get(s).copied()
    }
}

impl SymbolMap for HashMap<String, u32> {
    fn get(&self, s: &str) -> Option<u32> {
        self.get(s).copied()
    }
}
