use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use fairqueue::{FairGroup, FairQueue};

#[derive(Debug)]
struct Event {
    user_id: u32,
}

impl FairGroup for Event {
    #[inline(always)]
    fn is_same_group(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

fn make_events(group_count: usize, items_per_group: usize) -> Vec<Event> {
    (0..group_count)
        .flat_map(|group| {
            (0..items_per_group).map(move |_| Event {
                user_id: group as u32,
            })
        })
        .collect()
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    for &groups in &[4usize, 16, 64] {
        let events = make_events(groups, (64 / groups.max(1)).max(1));
        let references: Vec<&Event> = events.iter().collect();

        group.bench_function(BenchmarkId::from_parameter(groups), |b| {
            b.iter_batched(
                FairQueue::<Event>::new,
                |mut queue| {
                    for event in &references {
                        queue.insert(*event);
                    }
                    queue
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_round_robin(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_robin");
    let events = make_events(16, 16);
    let references: Vec<&Event> = events.iter().collect();

    group.bench_function("pop_cycle", |b| {
        b.iter_batched(
            || {
                let mut queue = FairQueue::<Event>::new();
                for event in &references {
                    queue.insert(*event);
                }
                queue
            },
            |mut queue| {
                while queue.pop().is_some() {}
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(benches, bench_insert, bench_round_robin);
criterion_main!(benches);
