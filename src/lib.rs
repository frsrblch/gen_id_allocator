pub use ref_cast::RefCast;
pub use static_assertions::assert_impl_one;

pub use alloc_gen::{AllocGen, ArenaGen};
pub use allocator::{Allocator, RangeAllocator};
pub use id::Id;
pub use range::IdRange;
pub use traits::*;
pub use valid::Valid;

#[cfg(feature = "untyped")]
pub mod untyped {
    pub use crate::alloc_gen::{UntypedAllocGen, UntypedArenaGen};
    pub use crate::allocator::UntypedAllocator;
    pub use crate::id::UntypedId;
    pub use crate::range::UntypedIdRange;
}

mod alloc_gen;
mod allocator;
mod gen;
mod id;
pub mod range;
mod traits;
mod valid;
