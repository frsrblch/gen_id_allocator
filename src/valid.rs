use crate::{Id, IdRange, ValidId, ValidRange};
use iter_context::ContextualIterator;
use ref_cast::RefCast;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Valid<'valid, T> {
    pub value: T,
    valid: PhantomData<&'valid ()>,
}

#[cfg(feature = "assert_valid")]
impl<'valid, T> RefCast for Valid<'valid, T> {
    type From = T;

    fn ref_cast(from: &Self::From) -> &Self {
        let ptr = from as *const Self::From as *const Self;
        unsafe { &*ptr }
    }

    fn ref_cast_mut(from: &mut Self::From) -> &mut Self {
        let ptr = from as *mut Self::From as *mut Self;
        unsafe { &mut *ptr }
    }
}

impl<'valid, T> Valid<'valid, T> {
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Valid {
            value,
            valid: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn new_ref(value: &T) -> &Valid<'valid, T> {
        let ptr = value as *const T as *const Self;
        unsafe { &*ptr }
    }

    #[inline]
    pub(crate) fn new_mut(value: &mut T) -> &mut Valid<'valid, T> {
        let ptr = value as *mut T as *mut Self;
        unsafe { &mut *ptr }
    }

    #[cfg(feature = "assert_valid")]
    #[inline]
    pub fn assert(value: T) -> Self {
        Valid::new(value)
    }

    #[deprecated]
    #[cfg(feature = "assert_valid")]
    #[inline]
    pub fn assert_ref(value: &T) -> &Self {
        Valid::ref_cast(value)
    }

    #[deprecated]
    #[cfg(feature = "assert_valid")]
    #[inline]
    pub fn assert_mut(value: &mut T) -> &mut Self {
        Valid::ref_cast_mut(value)
    }

    #[inline]
    pub fn from<U>(valid: Valid<'valid, U>) -> Self
    where
        T: From<U>,
    {
        Valid::new(T::from(valid.value))
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> Valid<'valid, U> {
        Valid::new(f(self.value))
    }
}

impl<'valid, T: Copy> Valid<'valid, &T> {
    #[inline]
    pub fn copied(&self) -> Valid<'valid, T> {
        Valid::new(*self.value)
    }
}

impl<'valid, Arena> ValidId for Valid<'valid, Id<Arena>> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        self.value.index()
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        self.value
    }
}

impl<'valid, Arena> ValidId for Valid<'valid, &'_ Id<Arena>> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        self.value.index()
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        *self.value
    }
}

impl<'valid, Arena> ValidId for &'_ Valid<'valid, Id<Arena>> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        self.value.index()
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        self.value
    }
}

impl<'valid, Arena> ValidId for &'_ Valid<'valid, &Id<Arena>> {
    type Arena = Arena;

    #[inline]
    fn index(self) -> usize {
        self.value.index()
    }

    #[inline]
    fn id(self) -> Id<Arena> {
        *self.value
    }
}

impl<'valid, IntoIter> IntoIterator for Valid<'valid, IntoIter>
where
    IntoIter: IntoIterator,
{
    type Item = Valid<'valid, IntoIter::Item>;
    type IntoIter = ValidIter<'valid, IntoIter::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ValidIter::new(self.value.into_iter())
    }
}

impl<'a, 'valid, IntoIter> IntoIterator for &'a Valid<'valid, IntoIter>
where
    &'a IntoIter: IntoIterator,
{
    type Item = Valid<'valid, <&'a IntoIter as IntoIterator>::Item>;
    type IntoIter = ValidIter<'valid, <&'a IntoIter as IntoIterator>::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ValidIter::new(self.value.into_iter())
    }
}

impl<'valid, IntoIter, Context> ContextualIterator for Valid<'valid, IntoIter>
where
    IntoIter: ContextualIterator<Context = Context>,
{
    type Context = Context;
}

impl<'valid, Id: ValidId, Indexable: Index<Id>> Index<Id> for Valid<'valid, Indexable>
where
    Indexable::Output: Sized,
{
    type Output = Valid<'valid, Indexable::Output>;

    #[inline]
    fn index(&self, index: Id) -> &Self::Output {
        Valid::new_ref(self.value.index(index))
    }
}

impl<'valid, Id: ValidId, IndexableMut: IndexMut<Id>> IndexMut<Id> for Valid<'valid, IndexableMut>
where
    IndexableMut::Output: Sized,
{
    #[inline]
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        Valid::new_mut(self.value.index_mut(index))
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, RefCast)]
pub struct ValidIter<'valid, Iter> {
    iter: Iter,
    valid: PhantomData<&'valid ()>,
}

impl<'valid, Iter> ValidIter<'valid, Iter> {
    pub fn new(iter: Iter) -> Self {
        ValidIter {
            iter,
            valid: Default::default(),
        }
    }
}

impl<'valid, Iter: Iterator> Iterator for ValidIter<'valid, Iter> {
    type Item = Valid<'valid, Iter::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Valid::new)
    }
}

impl<'valid, Arena> ValidRange for Valid<'valid, IdRange<Arena>> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        self.value
    }
}

impl<'valid, Arena> ValidRange for Valid<'valid, &IdRange<Arena>> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        *self.value
    }
}

impl<'valid, Arena> ValidRange for &Valid<'valid, IdRange<Arena>> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        self.value
    }
}

impl<'valid, Arena> ValidRange for &Valid<'valid, &IdRange<Arena>> {
    type Arena = Arena;
    #[inline]
    fn range(self) -> IdRange<Arena> {
        *self.value
    }
}
