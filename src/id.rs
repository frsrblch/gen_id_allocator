use crate::gen::Gen;
use crate::{Fixed, ValidId};
use force_derive::*;
use ref_cast::RefCast;
use std::cmp::Ordering;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct UntypedId {
    pub index: u32,
    pub(crate) gen: Gen,
}

impl PartialEq for UntypedId {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index) & self.gen.eq(&other.gen)
    }
}

impl Eq for UntypedId {}

impl UntypedId {
    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn first(index: usize) -> Self {
        UntypedId {
            index: index as u32,
            gen: Gen::default(),
        }
    }

    #[cfg(not(feature = "id_creation"))]
    #[inline]
    pub(crate) fn first(index: usize) -> Self {
        UntypedId {
            index: index as u32,
            gen: Gen::default(),
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index as usize
    }

    #[inline]
    pub fn increment_gen(&mut self) {
        self.gen = self.gen.next();
    }

    #[inline]
    pub(crate) fn bits(self) -> u64 {
        (self.index as u64) << 32 | self.gen.get() as u64
    }
}

#[repr(transparent)]
#[derive(Debug, ForceCopy, ForceClone, ForceEq, ForcePartialEq, ForceHash, RefCast)]
pub struct Id<Arena> {
    pub untyped: UntypedId,
    marker: PhantomData<*const Arena>,
}

impl<Arena> Ord for Id<Arena> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.untyped.cmp(&other.untyped)
    }
}

impl<Arena> PartialOrd for Id<Arena> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Arena> Id<Arena> {
    #[cfg(not(feature = "id_creation"))]
    #[inline]
    pub(crate) fn first(index: usize) -> Self {
        Id::new(UntypedId::first(index))
    }

    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn first(index: usize) -> Self {
        Self::new(UntypedId {
            index: index as u32,
            gen: Gen::default(),
        })
    }

    #[cfg(not(feature = "id_creation"))]
    #[inline]
    pub(crate) fn new(id: UntypedId) -> Self {
        Id {
            untyped: id,
            marker: PhantomData,
        }
    }

    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn new(id: UntypedId) -> Self {
        Id {
            untyped: id,
            marker: PhantomData,
        }
    }

    #[inline]
    pub(super) fn increment_gen(&mut self) {
        self.untyped.increment_gen();
    }

    #[inline]
    pub fn index(self) -> usize {
        self.untyped.index()
    }

    #[inline]
    pub(crate) fn gen(&self) -> Gen {
        self.untyped.gen
    }
}

impl<Arena: Fixed> ValidId for Id<Arena> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        Id::index(self)
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        self
    }
}

impl<Arena: Fixed> ValidId for &Id<Arena> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        Id::index(*self)
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_and_alignment() {
        use std::mem::{align_of, size_of};

        assert_eq!(size_of::<UntypedId>(), size_of::<Id<()>>());
        assert_eq!(align_of::<UntypedId>(), align_of::<Id<()>>());
    }
}
