# SpacedQueue

SpacedQueue is a Rust `no_std` (`alloc`) library that implements a spatially distancing queue, which ensures that its items are equitably interleaved. This data structure enforces round-robin scheduling semantics, guaranteeing that items associated with the same key are spaced apart maximally during processing.

The crate spacedqueue provides a generic key-value API with constraint on keys implementing total ordering (`Ord`). Presented algorithm leverages `BTreeMap` for key management and `VecDeque` for reduced overhead during queue operations.

## Usage

```Rust
use spacedqueue::SpacedQueue;

fn main() {
    let mut queue = SpacedQueue::new();

    queue.insert(&1, &"A");
    queue.insert(&1, &"B");
    queue.insert(&2, &"C");

    assert_eq!(queue.pop(), Some(&"A"));
    assert_eq!(queue.pop(), Some(&"C"));
    assert_eq!(queue.pop(), Some(&"B"));
    assert_eq!(queue.pop(), None);
}
```

## Benchmarking Results

Empirical benchmark demonstrates that SpacedQueue outperforms multi-deque fair queuing implementation based on HashMap with interleaving, which is theoretically expected to have lesser O complexity due to use of hash-based lookups. The following results illustrate performance for processing 50,000 items:

```
Fair Queue Comparison/SpacedQueue/50000
                        time:   [946.91 µs 954.83 µs 963.05 µs]

Fair Queue Comparison/HashMapFairQueue/50000
                        time:   [1.6140 ms 1.6226 ms 1.6319 ms]
```

## API Overview

### `SpacedQueue`

- `new() -> SpacedQueue` - Constructs a new, empty instance of the queue.

- `insert(&mut self, key: &K, value: &V)` - Adds a value to the queue under the specified key, dynamically updating group management.

- `pop(&mut self) -> Option<&V>` - Removes and returns the next item in the queue, adhering to round-robin scheduling.

- `peek(&self) -> Option<&&V>` - Returns a reference to the next item in the queue without removing it.

## License

SpacedQueue is distributed under the [BSD Zero Clause License](LICENSE).
