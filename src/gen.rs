use std::num::NonZeroU32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct Gen(NonZeroU32);

impl Default for Gen {
    fn default() -> Self {
        Self::new(1).unwrap()
    }
}

impl Gen {
    pub fn new(gen: u32) -> Option<Self> {
        NonZeroU32::new(gen).map(Self)
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }

    pub fn next(&self) -> Self {
        let gen = self.0.get().wrapping_add(1);
        Self::new(gen).unwrap_or_default()
    }
}
