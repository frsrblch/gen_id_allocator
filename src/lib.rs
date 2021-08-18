#![allow(dead_code)]

pub use static_assertions::assert_impl_one;

pub use alloc_gen::*;
pub use allocator::*;
pub use id::*;
pub use range::{IdRange, UntypedIdRange};
pub use traits::*;
pub use valid::*;

mod alloc_gen;
mod allocator;
mod gen;
mod id;
pub mod range;
mod traits;
mod valid;
