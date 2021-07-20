use crate::id::UntypedId;
use crate::Id;
use fnv::FnvHasher;
use my_derives::*;
use ref_cast::RefCast;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug, Clone, Default)]
pub struct UntypedArenaGen(u64);

impl UntypedArenaGen {
    #[inline]
    pub(crate) fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        let mut hasher = FnvHasher::with_key(self.0);
        id.hash(&mut hasher);
        self.0 = hasher.finish();
    }
}

impl From<&UntypedAllocGen> for u64 {
    fn from(value: &UntypedAllocGen) -> Self {
        value.0
    }
}

impl<Arena> From<&AllocGen<Arena>> for u64 {
    fn from(value: &AllocGen<Arena>) -> Self {
        value.0 .0
    }
}

impl From<&UntypedArenaGen> for u64 {
    fn from(value: &UntypedArenaGen) -> Self {
        value.0
    }
}

impl<Arena> From<&ArenaGen<Arena>> for u64 {
    fn from(value: &ArenaGen<Arena>) -> Self {
        value.0 .0
    }
}

impl PartialEq<UntypedAllocGen> for UntypedArenaGen {
    #[inline]
    fn eq(&self, other: &UntypedAllocGen) -> bool {
        self.0.eq(&other.0)
    }
}

#[repr(transparent)]
#[derive(Debug, MyClone, MyDefault, RefCast)]
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
    pub(crate) fn increment_gen(&mut self, id: UntypedId) {
        let mut hasher = FnvHasher::with_key(self.0);
        id.hash(&mut hasher);
        self.0 = hasher.finish();
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
