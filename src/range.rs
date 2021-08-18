use super::*;
use force_derive::*;
use std::marker::PhantomData;
use std::ops::Range;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct UntypedIdRange {
    start: usize,
    end: usize,
}

impl UntypedIdRange {
    #[cfg(not(feature = "id_creation"))]
    #[inline]
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn range(self) -> Range<usize> {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}

impl IntoIterator for UntypedIdRange {
    type Item = UntypedId;
    type IntoIter = UntypedIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        UntypedIter {
            range: self.start..self.end,
        }
    }
}

#[derive(Debug, ForceDefault, ForceCopy, ForceClone, ForceEq, ForcePartialEq, ForceHash)]
pub struct IdRange<Arena> {
    range: UntypedIdRange,
    arena: PhantomData<*const Arena>,
}

impl<Arena: Fixed> From<UntypedIdRange> for IdRange<Arena> {
    #[inline]
    fn from(range: UntypedIdRange) -> Self {
        Self {
            range,
            arena: PhantomData,
        }
    }
}

impl<Arena: Fixed> IdRange<Arena> {
    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self::from(UntypedIdRange::new(start, end))
    }
}

impl<Arena> IdRange<Arena> {
    #[inline]
    pub fn range(self) -> UntypedIdRange {
        self.range
    }
}

impl<Arena> IntoIterator for IdRange<Arena> {
    type Item = Id<Arena>;
    type IntoIter = Iter<Arena>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.range.into_iter(),
            arena: PhantomData,
        }
    }
}

#[derive(Debug, ForceClone)]
pub struct UntypedIter {
    range: Range<usize>,
}

impl Iterator for UntypedIter {
    type Item = UntypedId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(UntypedId::first)
    }
}

#[derive(Debug, ForceClone)]
pub struct Iter<Arena> {
    iter: UntypedIter,
    arena: PhantomData<*const Arena>,
}

impl<Arena> Iterator for Iter<Arena> {
    type Item = Id<Arena>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Id::new)
    }
}
