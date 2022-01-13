use crate::id::UntypedId;
use crate::Id;
use force_derive::*;
use ref_cast::RefCast;
use std::marker::PhantomData;

#[inline]
fn wrapping_shl_bit_xor(hash: &mut u64, id: UntypedId) {
    let bits = id.bits();
    *hash <<= 1;
    *hash ^= bits;
}

#[derive(Debug, Clone, Default)]
pub struct UntypedArenaGen(u64);

impl UntypedArenaGen {
    #[inline]
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        wrapping_shl_bit_xor(&mut self.0, id);
    }

    #[inline]
    pub(crate) fn update(&mut self, before: &UntypedAllocGen, after: &UntypedAllocGen) {
        assert_eq!(self, before);
        self.0 = after.0;
    }
}

impl PartialEq<UntypedAllocGen> for UntypedArenaGen {
    #[inline]
    fn eq(&self, other: &UntypedAllocGen) -> bool {
        self.0.eq(&other.0)
    }
}

#[derive(Debug, Default)]
pub struct UntypedAllocGen(u64);

impl UntypedAllocGen {
    #[inline]
    pub(crate) fn clone(&self) -> Self {
        Self(self.0)
    }

    #[inline]
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        wrapping_shl_bit_xor(&mut self.0, id);
    }
}

#[repr(transparent)]
#[derive(Debug, ForceClone, ForceDefault, RefCast)]
pub struct ArenaGen<Arena>(UntypedArenaGen, PhantomData<Arena>);

impl<Arena> ArenaGen<Arena> {
    #[inline]
    pub fn increment_gen(&mut self, id: Id<Arena>) {
        self.0.increment_gen(id.untyped);
    }

    #[inline]
    pub(crate) fn update(&mut self, before: &AllocGen<Arena>, after: &AllocGen<Arena>) {
        self.0.update(&before.0, &after.0);
    }
}

impl<Arena> PartialEq<AllocGen<Arena>> for ArenaGen<Arena> {
    #[inline]
    fn eq(&self, other: &AllocGen<Arena>) -> bool {
        self.0.eq(&other.0)
    }
}

/// Does not impl Copy/Clone so it must be borrowed from the Allocator
#[repr(transparent)]
#[derive(Debug, RefCast)]
pub struct AllocGen<Arena>(UntypedAllocGen, PhantomData<Arena>);

impl<Arena> AllocGen<Arena> {
    pub(crate) fn new(gen: UntypedAllocGen) -> Self {
        Self(gen, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_gen_alloc_gen_eq() {
        let mut arena_gen = UntypedArenaGen::default();
        let mut alloc_gen = UntypedAllocGen::default();
        let id = UntypedId::first(1);

        assert!(arena_gen.eq(&alloc_gen));

        arena_gen.increment_gen(id);

        assert!(arena_gen.ne(&alloc_gen));

        alloc_gen.increment_gen(id);

        assert!(arena_gen.eq(&alloc_gen));
    }
}
