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

    #[inline]
    pub fn extend(&mut self, id: UntypedId) {
        debug_assert_eq!(id.gen, crate::gen::Gen::default());
        let index = id.index();

        if self.end == index {
            self.end += 1;
        } else if self.start == index + 1 {
            self.start -= 1;
        } else if *self == UntypedIdRange::default() {
            *self = UntypedIdRange::from(id)
        } else {
            panic!("Invalid Id index")
        }
    }
}

impl From<UntypedId> for UntypedIdRange {
    #[inline]
    fn from(id: UntypedId) -> Self {
        debug_assert_eq!(id.gen, crate::gen::Gen::default());

        let start = id.index as usize;
        let end = start + 1;

        UntypedIdRange { start, end }
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

impl<Arena: Fixed> From<Id<Arena>> for IdRange<Arena> {
    #[inline]
    fn from(id: Id<Arena>) -> Self {
        UntypedIdRange::from(id.untyped).into()
    }
}

impl<Arena: Fixed> IdRange<Arena> {
    #[cfg(feature = "id_creation")]
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self::from(UntypedIdRange::new(start, end))
    }

    #[inline]
    pub fn range(self) -> UntypedIdRange {
        self.range
    }

    #[inline]
    pub fn extend<V: ValidId<Arena = Arena>>(&mut self, id: V) {
        self.range.extend(id.id().untyped)
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

impl<Arena: Fixed> ValidRange for IdRange<Arena> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        self
    }
}

impl<Arena: Fixed> ValidRange for &IdRange<Arena> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        *self
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

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct Fixed;
    fixed_id!(Fixed);

    #[test]
    fn extend_default() {
        let mut range = IdRange::<Fixed>::default();
        let id = Id::first(3);

        range.extend(id);

        assert_eq!(IdRange::from(id), range);
    }

    #[test]
    fn extend_end() {
        let id2 = Id::first(2);
        let id3 = Id::first(3);
        let mut range = IdRange::<Fixed>::from(id2);

        range.extend(id3);

        assert_eq!(IdRange::new(2, 4), range);
    }

    #[test]
    fn extend_start() {
        let id2 = Id::first(2);
        let id3 = Id::first(3);
        let mut range = IdRange::<Fixed>::from(id3);

        range.extend(id2);

        assert_eq!(IdRange::new(2, 4), range);
    }
}
