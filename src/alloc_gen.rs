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
    pub(crate) fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        wrapping_shl_bit_xor(&mut self.0, id);
    }
}

impl PartialEq<UntypedAllocGen> for UntypedArenaGen {
    #[inline]
    fn eq(&self, other: &UntypedAllocGen) -> bool {
        self.0.eq(&other.0)
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
}

impl<Arena> PartialEq<AllocGen<Arena>> for ArenaGen<Arena> {
    #[inline]
    fn eq(&self, other: &AllocGen<Arena>) -> bool {
        self.0.eq(&other.0)
    }
}

#[derive(Debug, Default)]
pub struct UntypedAllocGen(u64);

impl UntypedAllocGen {
    #[inline]
    pub(crate) fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub(crate) fn clone(&self) -> Self {
        Self(self.0)
    }

    #[inline]
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        wrapping_shl_bit_xor(&mut self.0, id);
    }
}

/// Does not impl Copy/Clone so it must be borrowed from the Allocator
#[repr(transparent)]
#[derive(Debug, RefCast)]
pub struct AllocGen<Arena>(UntypedAllocGen, PhantomData<Arena>);

impl<Arena> AllocGen<Arena> {
    /// Can only create in crate, only source of AllocGen is to borrow from an Allocator
    #[inline]
    pub(crate) fn new() -> Self {
        Self(UntypedAllocGen::new(), PhantomData)
    }

    #[inline]
    pub(crate) fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }

    #[inline]
    pub(crate) fn increment_gen(&mut self, id: Id<Arena>) {
        self.0.increment_gen(id.untyped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_gen_alloc_gen_eq() {
        let mut arena_gen = ArenaGen::default();
        let mut alloc_gen = AllocGen::new();
        let id = Id::<()>::first(1);

        assert!(arena_gen.eq(&alloc_gen));

        arena_gen.increment_gen(id);

        assert!(arena_gen.ne(&alloc_gen));

        alloc_gen.increment_gen(id);

        assert!(arena_gen.eq(&alloc_gen));
    }
}
