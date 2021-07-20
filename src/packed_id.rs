use crate::gen::Gen;
use my_derives::*;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::num::NonZeroU64;

#[derive(Debug, MyCopy, MyClone, MyEq, MyPartialEq, MyHash)]
pub struct Id<Arena> {
    bits: NonZeroU64,
    marker: PhantomData<Arena>,
}

impl<Arena> Ord for Id<Arena> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bits.cmp(&other.bits)
    }
}

impl<Arena> PartialOrd for Id<Arena> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Arena> Id<Arena> {
    pub(crate) fn first(index: usize) -> Self {
        Self::new(index, Gen::default())
    }

    pub(super) fn increment_gen(&mut self) {
        let gen = self.gen().next();
        *self = Self::new(self.index(), gen);
    }

    pub(crate) fn new(index: usize, gen: Gen) -> Self {
        let index_bits: u64 = u64::try_from(index).unwrap() << Gen::SIZE_IN_BITS;
        let gen_bits: u64 = gen.get_bits();

        let bits: u64 = index_bits | gen_bits;

        // SAFETY: Gen is based on a non-zero integer, so its bits will never be zero
        let bits = unsafe { NonZeroU64::new_unchecked(bits) };

        Self::from_bits(bits)
    }

    fn from_bits(bits: NonZeroU64) -> Self {
        Self {
            bits,
            marker: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        let index = self.index_u64();

        usize::try_from(index).unwrap()
    }

    pub(crate) fn index_u32(&self) -> u32 {
        let index = self.index_u64();

        u32::try_from(index).unwrap()
    }

    fn index_u64(&self) -> u64 {
        self.bits.get() >> Gen::SIZE_IN_BITS
    }

    pub(crate) fn gen(&self) -> Gen {
        let gen = self.bits.get() & Gen::MASK;

        u32::try_from(gen).ok().and_then(Gen::new).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_packing() {
        let id = Id::<()>::new(2, Gen::new(3).unwrap());

        assert_eq!(2, id.index());
        assert_eq!(3, id.gen().get());
    }
}
