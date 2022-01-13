use crate::alloc_gen::{AllocGen, UntypedAllocGen};
use crate::gen::Gen;
use crate::id::UntypedId;
use crate::id::*;
use crate::range::{IdRange, UntypedIdRange};
use crate::valid::Valid;
use crate::{ArenaGen, Fixed, Validator};
use force_derive::*;
use nonmax::NonMaxU32;
use ref_cast::RefCast;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct UntypedAllocator {
    entries: Vec<Entry>,
    next_dead: Option<NonMaxU32>,
    gen: UntypedAllocGen,
}

#[derive(Debug)]
enum Entry {
    Dead(Dead),
    Alive(UntypedId),
}

#[derive(Debug)]
struct Dead {
    next_dead: Option<NonMaxU32>,
    gen: Gen,
}

impl Entry {
    fn dead(&self) -> Option<&Dead> {
        match self {
            Entry::Dead(dead) => Some(dead),
            Entry::Alive(_) => None,
        }
    }

    fn alive(&self) -> Option<UntypedId> {
        match self {
            Entry::Dead(_) => None,
            Entry::Alive(id) => Some(*id),
        }
    }
}

impl UntypedAllocator {
    #[inline]
    pub fn create(&mut self) -> UntypedId {
        self.reuse_index().unwrap_or_else(|| self.create_new())
    }

    #[inline]
    pub fn create_range(&mut self, len: usize) -> UntypedIdRange {
        let start = self.entries.len();
        let end = start + len;
        for _ in 0..len {
            // guarantees that Ids are contiguous at the end of `self.entries`
            let _ = self.create_new();
        }
        UntypedIdRange::new(start, end)
    }

    #[inline]
    fn reuse_index(&mut self) -> Option<UntypedId> {
        let index = self.next_dead?.get();
        let entry = self.entries.get_mut(index as usize)?;
        let &Dead { next_dead, gen } = entry.dead()?;

        self.next_dead = next_dead;

        let id = UntypedId { index, gen };
        *entry = Entry::Alive(id);
        Some(id)
    }

    #[inline]
    fn create_new(&mut self) -> UntypedId {
        let index = self.entries.len();
        let id = UntypedId::first(index);
        self.entries.push(Entry::Alive(id));
        id
    }

    #[inline]
    pub fn kill(&mut self, id: UntypedId) -> bool {
        if let Some(entry) = self.entries.get_mut(id.index()) {
            if matches!(entry, Entry::Alive(living) if *living == id) {
                *entry = Entry::Dead(Dead {
                    next_dead: self.next_dead,
                    gen: id.gen.next(),
                });
                self.next_dead = NonMaxU32::new(id.index);
                self.gen.increment_gen(id);

                return true;
            }
        }

        false
    }

    #[inline]
    pub fn is_alive(&self, id: UntypedId) -> bool {
        let entry = self.entries.get(id.index());
        matches!(entry, Some(Entry::Alive(living)) if *living == id)
    }

    #[inline]
    pub fn ids(&self) -> impl Iterator<Item = UntypedId> + '_ {
        self.entries.iter().filter_map(Entry::alive)
    }
}

#[repr(transparent)]
#[derive(Debug, ForceDefault, RefCast)]
pub struct Allocator<Arena> {
    untyped: UntypedAllocator,
    arena: PhantomData<Arena>,
}

impl<Arena> Allocator<Arena> {
    #[inline]
    pub fn create(&mut self) -> Valid<Id<Arena>> {
        Valid::new(self.create_id())
    }

    #[inline]
    fn create_id(&mut self) -> Id<Arena> {
        Id::new(self.untyped.create())
    }

    #[inline]
    pub fn kill(&mut self, id: Id<Arena>) -> bool {
        self.untyped.kill(id.untyped)
    }

    /// Drains the Vec, kills all the Ids, and filters out any duplicate or invalid Ids
    /// Returns a Killed type for the purpose of notifying other arenas of their deletion
    #[inline]
    #[must_use]
    pub fn kill_multiple(&mut self, ids: &mut Vec<Id<Arena>>) -> Killed<Arena> {
        // Take gen value before any Ids are killed
        let start = AllocGen::new(self.untyped.gen.clone());

        // Filters out dead Ids and any duplicate values
        let ids = ids.drain(..).filter(|id| self.kill(*id)).collect();

        // Take gen value after Ids are killed
        let end = AllocGen::new(self.untyped.gen.clone());

        Killed {
            ids: Valid::new(ids),
            before: start,
            after: end,
        }
    }

    #[inline]
    pub fn is_alive(&self, id: Id<Arena>) -> bool {
        self.untyped.is_alive(id.untyped)
    }

    #[inline]
    pub fn create_only<'valid>(&'valid mut self) -> &mut CreateOnly<'valid, Arena> {
        RefCast::ref_cast_mut(self)
    }

    #[inline]
    pub fn validate(&self, id: Id<Arena>) -> Option<Valid<Id<Arena>>> {
        Validator::validate(&self, id)
    }

    #[inline]
    pub fn ids<'valid>(&'valid self) -> impl Iterator<Item = Valid<'valid, Id<Arena>>> + '_ {
        self.untyped
            .entries
            .iter()
            .filter_map(Entry::alive)
            .map(Id::new)
            .map(Valid::new)
    }
}

impl<Arena: Fixed> Allocator<Arena> {
    #[inline]
    pub fn create_range(&mut self, len: usize) -> IdRange<Arena> {
        let range = self.untyped.create_range(len);
        IdRange::from(range)
    }
}

impl<Arena> AsRef<AllocGen<Arena>> for Allocator<Arena> {
    #[inline]
    fn as_ref(&self) -> &AllocGen<Arena> {
        RefCast::ref_cast(&self.untyped.gen)
    }
}

// Must implement for an Allocator reference so that there is a lifetime
// for the resulting value to inherit
impl<'valid, Arena> Validator<'valid, Arena> for &'valid Allocator<Arena> {
    #[inline]
    fn validate(&self, id: Id<Arena>) -> Option<Valid<'valid, Id<Arena>>> {
        if self.is_alive(id) {
            Some(Valid::new(id))
        } else {
            None
        }
    }
}

impl<'valid, Arena> Validator<'valid, Arena> for &'valid mut Allocator<Arena> {
    #[inline]
    fn validate(&self, id: Id<Arena>) -> Option<Valid<'valid, Id<Arena>>> {
        if self.is_alive(id) {
            Some(Valid::new(id))
        } else {
            None
        }
    }
}

#[repr(transparent)]
#[derive(Debug, RefCast)]
pub struct CreateOnly<'valid, Arena> {
    allocator: Allocator<Arena>,
    valid: PhantomData<&'valid ()>,
}

impl<'valid, Arena> CreateOnly<'valid, Arena> {
    #[inline]
    pub fn create(&mut self) -> Valid<'valid, Id<Arena>> {
        Valid::new(self.allocator.create_id())
    }

    #[inline]
    pub fn is_alive(&self, id: Id<Arena>) -> bool {
        self.allocator.is_alive(id)
    }

    #[inline]
    pub fn ids(&self) -> impl Iterator<Item = Valid<'valid, Id<Arena>>> + '_ {
        self.allocator
            .untyped
            .entries
            .iter()
            .filter_map(Entry::alive)
            .map(Id::new)
            .map(Valid::new)
    }
}

impl<Arena> AsRef<AllocGen<Arena>> for CreateOnly<'_, Arena> {
    #[inline]
    fn as_ref(&self) -> &AllocGen<Arena> {
        self.allocator.as_ref()
    }
}

impl<'valid, Arena> Validator<'valid, Arena> for CreateOnly<'valid, Arena> {
    #[inline]
    fn validate(&self, id: Id<Arena>) -> Option<Valid<'valid, Id<Arena>>> {
        self.is_alive(id).then(|| Valid::new(id))
    }
}

impl<'a, 'valid, Arena> Validator<'valid, Arena> for &'a mut CreateOnly<'valid, Arena> {
    #[inline]
    fn validate(&self, id: Id<Arena>) -> Option<Valid<'valid, Id<Arena>>> {
        self.is_alive(id).then(|| Valid::new(id))
    }
}

/// A list of valid, unique Ids that have been killed.
/// Includes before and after allocator generations for validating and updating ArenaGen values  
#[derive(Debug)]
pub struct Killed<'v, Arena> {
    ids: Valid<'v, Vec<Id<Arena>>>,
    before: AllocGen<Arena>,
    after: AllocGen<Arena>,
}

impl<'v, Arena> Killed<'v, Arena> {
    pub fn update_gen(&self, gen: &mut ArenaGen<Arena>) {
        gen.update(&self.before, &self.after);
    }

    pub fn iter(&self) -> impl Iterator<Item = Valid<'v, &Id<Arena>>> {
        (&self.ids).into_iter()
    }
}

impl<'a, 'v, Arena> IntoIterator for &'a Killed<'v, Arena> {
    type Item = Valid<'v, &'a Id<Arena>>;
    type IntoIter = crate::valid::ValidIter<'v, std::slice::Iter<'a, Id<Arena>>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.ids).into_iter()
    }
}

#[derive(Debug, ForceDefault, ForceClone)]
pub struct RangeAllocator<Arena> {
    next: usize,
    arena: PhantomData<*const Arena>,
}

impl<Arena: Fixed> RangeAllocator<Arena> {
    #[inline]
    pub fn create(&mut self, len: usize) -> IdRange<Arena> {
        let start = self.next;
        self.next += len;
        let end = self.next;
        IdRange::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_id() {
        let mut allocator = Allocator::<()>::default();

        let id = allocator.create_id();

        assert_eq!(0, id.index());
        assert_eq!(1, id.gen().get());
    }

    #[test]
    fn create_id_1() {
        let mut allocator = Allocator::<()>::default();

        let _ = allocator.create_id();
        let id = allocator.create_id();

        assert_eq!(1, id.index());
        assert_eq!(1, id.gen().get());
    }

    #[test]
    fn is_alive_given_living() {
        let mut allocator = Allocator::<()>::default();

        let id = allocator.create_id();

        assert!(allocator.is_alive(id));
    }

    #[test]
    fn is_alive_given_dead() {
        let mut allocator = Allocator::<()>::default();

        let id = allocator.create_id();
        allocator.kill(id);

        assert!(!allocator.is_alive(id));
    }

    #[test]
    fn reuse_index() {
        let mut allocator = Allocator::<()>::default();
        let id = allocator.create_id();
        allocator.kill(id);

        let id = allocator.create_id();

        assert_eq!(0, id.index());
        assert_eq!(2, id.gen().get());
    }

    #[test]
    fn reuse_multiple() {
        let mut allocator = Allocator::<()>::default();
        let id0 = allocator.create_id();
        let id1 = allocator.create_id();
        let id2 = allocator.create_id();

        // order matters for test
        allocator.kill(id0);
        allocator.kill(id1);
        allocator.kill(id2);

        let id2 = allocator.create_id();
        let id1 = allocator.create_id();
        let id0 = allocator.create_id();

        assert_eq!(0, id0.index());
        assert_eq!(2, id0.gen().get());

        assert_eq!(1, id1.index());
        assert_eq!(2, id1.gen().get());

        assert_eq!(2, id2.index());
        assert_eq!(2, id2.gen().get());
    }

    #[test]
    fn validate_allocator() {
        let mut allocator = Allocator::<()>::default();
        let id = allocator.create_id();

        let valid = (&allocator).validate(id).unwrap();

        // // uncomment to break compilation
        // let _ = allocator.kill(id);

        dbg!(valid.value);
    }

    #[test]
    fn validate_create_only() {
        let mut allocator = Allocator::<()>::default();
        let id = allocator.create_id();

        let create_only = allocator.create_only();

        let valid = create_only.validate(id).unwrap();

        let _new_id = create_only.create();

        dbg!(valid.value);
    }

    #[test]
    fn entry_size() {
        assert_eq!(12, std::mem::size_of::<Entry>());
    }

    #[test]
    fn create_range() {
        #[derive(Debug, Copy, Clone)]
        struct Fixed;
        crate::fixed_id!(Fixed);

        let mut alloc = Allocator::<Fixed>::default();
        let rows = vec![Fixed; 3];
        let range = alloc.create_range(rows.len());

        let ids = range.into_iter().collect::<Vec<_>>();

        assert_eq!(vec![Id::first(0), Id::first(1), Id::first(2)], ids);
    }

    #[test]
    fn range_allocator_create() {
        #[derive(Debug, Copy, Clone)]
        struct Fixed;
        crate::fixed_id!(Fixed);

        let mut allocator = RangeAllocator::<Fixed>::default();

        let range = allocator.create(3);
        assert_eq!(0, range.range.start);
        assert_eq!(3, range.len());

        let range = allocator.create(2);
        assert_eq!(3, range.range.start);
        assert_eq!(2, range.len());
    }

    #[test]
    fn kill_vec_given_live() {
        #[derive(Debug)]
        struct Dynamic;
        crate::dynamic_id!(Dynamic);

        let mut alloc = Allocator::<Dynamic>::default();

        let id = alloc.create().value;
        let mut ids = vec![id];

        let killed = alloc.kill_multiple(&mut ids);

        assert_eq!(vec![id], killed.ids.value);
    }

    #[test]
    fn kill_vec_given_dead_returns_empty() {
        #[derive(Debug)]
        struct Dynamic;
        crate::dynamic_id!(Dynamic);

        let mut alloc = Allocator::<Dynamic>::default();

        let id = alloc.create().value;
        let mut ids = vec![id];
        alloc.kill(id);

        let killed = alloc.kill_multiple(&mut ids);

        assert_eq!(Vec::<Id<Dynamic>>::new(), killed.ids.value);
    }

    #[test]
    fn kill_vec_given_duplicate_returns_single() {
        #[derive(Debug)]
        struct Dynamic;
        crate::dynamic_id!(Dynamic);

        let mut alloc = Allocator::<Dynamic>::default();

        let id = alloc.create().value;
        let mut ids = vec![id, id];

        let killed = alloc.kill_multiple(&mut ids);

        assert_eq!(vec![id], killed.ids.value);
    }

    #[test]
    fn kill_vec_borrow_test() {
        #[allow(dead_code)]
        fn borrow_alloc<'v, V: Validator<'v, Dynamic>>(_: V) {}

        #[derive(Debug)]
        struct Dynamic;
        crate::dynamic_id!(Dynamic);

        let mut alloc = Allocator::<Dynamic>::default();
        let killed = alloc.kill_multiple(&mut vec![]);

        // borrow_alloc(alloc);

        // uncomment to break compilation
        // let id = alloc.create().value;
        // alloc.kill(id);

        dbg!(killed);
    }
}
