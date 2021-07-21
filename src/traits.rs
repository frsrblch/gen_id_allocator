use crate::{Id, Valid};

pub trait ValidId: Copy {
    /// Type is used instead of a generic parameter so that it can be referenced by `MaybeValidId`
    type Arena;
    fn index(&self) -> usize;
    fn id(&self) -> Id<Self::Arena>;

    #[inline]
    fn valid<'valid>(&self) -> Valid<'valid, Id<Self::Arena>>
    where
        Self: 'valid,
    {
        Valid::new(self.id())
    }
}

#[test]
fn valid_test() {
    use crate::Validator;
    let mut alloc = crate::Allocator::<()>::default();
    let id = alloc.create().value;

    let create_only = alloc.create_only();
    let valid = create_only.validate(id).unwrap();
    let valid2 = valid.valid();

    // /// uncomment to break compilation
    // alloc.kill(id);

    dbg!("{:?}", valid2);
}

pub trait Fixed {}

#[macro_export]
macro_rules! fixed_id {
    ($t:ty) => {
        impl $crate::Fixed for $t {}
        $crate::assert_impl_one! { $t: $crate::Fixed, $crate::Dynamic }
    };
}

pub trait Dynamic {}

#[macro_export]
macro_rules! dynamic_id {
    ($t:ty) => {
        impl $crate::Dynamic for $t {}
        $crate::assert_impl_one! { $t: $crate::Fixed, $crate::Dynamic }
    };
}

/// Used by `gen_id_map_to` crate.
/// Defined here so that the blanket impls for `Option<V>` and `V` do not conflict,
/// which happens if `ValidId` is in the upstream crate.
pub trait MaybeValidId {
    type Arena;
    type Output: ValidId<Arena = Self::Arena>;
    fn try_valid(&self) -> Option<Self::Output>;
}

impl<V: ValidId> MaybeValidId for Option<V> {
    type Arena = V::Arena;
    type Output = V;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        *self
    }
}

impl<V: ValidId> MaybeValidId for &'_ Option<V> {
    type Arena = V::Arena;
    type Output = V;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        **self
    }
}

impl<V: ValidId> MaybeValidId for V {
    type Arena = V::Arena;
    type Output = V;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        Some(*self)
    }
}

impl<'valid, Arena> MaybeValidId for Valid<'valid, Option<Id<Arena>>> {
    type Arena = Arena;
    type Output = Valid<'valid, Id<Arena>>;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        self.value.map(Valid::new)
    }
}

impl<'a, 'valid, Arena> MaybeValidId for Valid<'valid, &'a Option<Id<Arena>>> {
    type Arena = Arena;
    type Output = Valid<'valid, &'a Id<Arena>>;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        self.value.as_ref().map(Valid::new)
    }
}

impl<'valid, Arena> MaybeValidId for &'_ Valid<'valid, Option<Id<Arena>>> {
    type Arena = Arena;
    type Output = Valid<'valid, Id<Arena>>;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        self.value.map(Valid::new)
    }
}

impl<'a, 'valid, Arena> MaybeValidId for &'_ Valid<'valid, &'a Option<Id<Arena>>> {
    type Arena = Arena;
    type Output = Valid<'valid, &'a Id<Arena>>;

    #[inline]
    fn try_valid(&self) -> Option<Self::Output> {
        self.value.as_ref().map(Valid::new)
    }
}
