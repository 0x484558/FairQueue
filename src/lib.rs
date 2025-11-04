#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod group;
mod queue;
mod stack;

pub use group::FairGroup;
pub use queue::FairQueue;
pub use stack::FairStack;
