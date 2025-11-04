#![no_std]

extern crate alloc;

mod group;
mod queue;
mod stack;

pub use group::FairGroup;
pub use queue::FairQueue;
pub use stack::FairStack;
