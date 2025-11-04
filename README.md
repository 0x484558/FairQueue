# FairQueue

[![CI](https://github.com/0x484558/fairqueue/actions/workflows/ci.yml/badge.svg)](https://github.com/0x484558/fairqueue/actions/workflows/ci.yml)
[![Docs](https://github.com/0x484558/fairqueue/actions/workflows/docs.yml/badge.svg)](https://github.com/0x484558/fairqueue/actions/workflows/docs.yml)
[![Benchmarks](https://github.com/0x484558/fairqueue/actions/workflows/benchmarks.yml/badge.svg)](https://github.com/0x484558/fairqueue/actions/workflows/benchmarks.yml)
[![Security Audit](https://github.com/0x484558/fairqueue/actions/workflows/audit.yml/badge.svg)](https://github.com/0x484558/fairqueue/actions/workflows/audit.yml)
[![Release](https://github.com/0x484558/fairqueue/actions/workflows/release.yml/badge.svg)](https://github.com/0x484558/fairqueue/actions/workflows/release.yml)

FairQueue is a Rust `no_std` (`alloc`) library that implements FIFO (queue) and LIFO (stack) data structures with equitable interleaving of groups of values. Such distancing allows amortized O(1) round-robin retrieval while storing values by reference and avoiding heavy moves. The optional `std` feature adds zero-cost conveniences for callers that want to collect iterator results.

## Example

```rust
use fairqueue::{FairGroup, FairQueue, FairStack};

#[derive(Debug, PartialEq)]
struct Event {
    user: &'static str,
    value: u32,
}

impl FairGroup for Event {
    fn is_same_group(&self, other: &Self) -> bool {
        self.user == other.user
    }
}

let a1 = Event { user: "alice", value: 1 };
let a2 = Event { user: "alice", value: 2 };
let b1 = Event { user: "bob", value: 9 };

let mut queue = FairQueue::new();
queue.insert(&a1);
queue.insert(&a2);
queue.insert(&b1);
assert_eq!(queue.pop(), Some(&a1));
assert_eq!(queue.pop(), Some(&b1));
assert_eq!(queue.pop(), Some(&a2));

let mut stack = FairStack::new();
stack.push(&a1);
stack.push(&a2);
stack.push(&b1);
assert_eq!(stack.pop(), Some(&a2));
assert_eq!(stack.pop(), Some(&b1));
assert_eq!(stack.pop(), Some(&a1));
```

## API Overview

- `FairGroup` trait - Defines group identity for items stored in either structure. Implementers usually rely on a key field or pointer identity so that comparisons remain near zero-cost.
- `FairQueue` - Implements FIFO queuing while keeping groups spatially separated. Core methods include `new`, `insert`, `pop`, `peek`, `len`, `is_empty`, and `group_count`. Observability stays cheap through the `group_heads` iterator, with `group_heads_vec` available when the `std` feature is enabled.
- `FairStack` - Implements LIFO queuing with the same fairness guarantees. The API mirrors the queue with `new`, `push`, `pop`, `peek`, `peek_group`, `len`, `is_empty`, and `group_count`. It also exposes `group_heads` and, under the `std` feature, `group_heads_vec` for eager collection.

## License

Licensed under the [BSD Zero Clause License](LICENSE).
