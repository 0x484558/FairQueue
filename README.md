# FairQueue

FairQueue is a Rust `no_std` (`alloc`) library that implements a fair queue through spatial distancing of similar values. The data structure presented in `fairqueue` crate ensures that groups of items are equitably interleaved with items from other groups and can be scheduled in a round-robin manner.

## Usage

```Rust
use fairqueue::{FairQueue, FairGroup};

#[derive(Debug, PartialEq)]
struct Event {
    timestamp: u32,
    user_id: &'static str,
}

impl FairGroup for Event {
    fn is_same_group(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

fn main() {
    let mut queue = FairQueue::new();

    queue.insert(&Event { timestamp: 1, user_id: "user1" });
    queue.insert(&Event { timestamp: 2, user_id: "user2" });
    queue.insert(&Event { timestamp: 3, user_id: "user1" });

    assert_eq!(queue.pop(), Some(&Event { timestamp: 1, user_id: "user1" }));
    assert_eq!(queue.pop(), Some(&Event { timestamp: 2, user_id: "user2" }));
    assert_eq!(queue.pop(), Some(&Event { timestamp: 3, user_id: "user1" }));
    assert_eq!(queue.pop(), None);
}
```

## API Overview

### `FairQueue`

- `new() -> FairQueue` - Constructs a new, empty instance of the queue.

- `insert(&mut self, value: &V: FairGroup)` - Adds a value to the queue, distancing it from values within the same group.

- `pop(&mut self) -> Option<&V>` - Removes and returns the next item in the queue, adhering to round-robin scheduling.

- `peek(&self) -> Option<&&V>` - Returns a reference to the next item in the queue without removing it.

## License

FairQueue is distributed under the [BSD Zero Clause License](LICENSE).
