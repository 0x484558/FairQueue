use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use spacedqueue::SpacedQueue;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

// HashMap-based fair queue with interleaving
struct HashMapFairQueue<'a, K: Eq + Hash + Ord, V: 'a> {
    queues: HashMap<&'a K, VecDeque<&'a V>>, // Maps keys to queues
    order: Vec<&'a K>, // Keeps round-robin order of keys
    pointer: usize,    // Tracks the current key for dequeuing
}

impl<'a, K: Eq + Hash + Ord, V: 'a> HashMapFairQueue<'a, K, V> {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            order: Vec::new(),
            pointer: 0,
        }
    }

    pub fn insert(&mut self, key: &'a K, value: &'a V) {
        let entry = self.queues.entry(key).or_insert_with(|| {
            self.order.push(key);
            VecDeque::new()
        });
        entry.push_back(value);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<&'a V> {
        if self.order.is_empty() {
            return None;
        }

        for _ in 0..self.order.len() {
            let key = self.order[self.pointer];
            if let Some(queue) = self.queues.get_mut(key) {
                if let Some(value) = queue.pop_front() {
                    if queue.is_empty() {
                        self.queues.remove(key);
                        self.order.remove(self.pointer);
                        if self.pointer >= self.order.len() {
                            self.pointer = 0;
                        }
                    } else {
                        self.pointer = (self.pointer + 1) % self.order.len();
                    }
                    return Some(value);
                }
            }
            self.pointer = (self.pointer + 1) % self.order.len();
        }

        None
    }
}

// Benchmarking code
fn benchmark_queues(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fair Queue Comparison");
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function(BenchmarkId::new("SpacedQueue", 50_000), |b| {
        b.iter(|| {
            let mut queue = SpacedQueue::new();
            let items: Vec<(usize, usize)> = (0..50_000)
                .map(|i| (i % 100, i))
                .collect();
            for (key, value) in &items {
                queue.insert(key, value);
            }
            while queue.pop().is_some() {}
        });
    });

    group.bench_function(BenchmarkId::new("HashMapFairQueue", 50_000), |b| {
        b.iter(|| {
            let mut queue = HashMapFairQueue::new();
            let items: Vec<(usize, usize)> = (0..50_000)
                .map(|i| (i % 100, i))
                .collect();
            for (key, value) in &items {
                queue.insert(key, value);
            }
            while queue.pop().is_some() {}
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_queues);
criterion_main!(benches);
