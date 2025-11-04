use alloc::{collections::VecDeque, vec::Vec};
use core::{ptr, slice};

use crate::FairGroup;

/// Spatially distancing fair queue.
/// First in, first out, ensuring that each group of similar values
/// is placed as far apart as possible.
///
/// ```
/// use fairqueue::{FairGroup, FairQueue};
///
/// #[derive(Debug, PartialEq)]
/// struct Event {
///     user_id: &'static str,
///     value: u32,
/// }
///
/// impl FairGroup for Event {
///     fn is_same_group(&self, other: &Self) -> bool {
///         self.user_id == other.user_id
///     }
/// }
///
/// let user_a_first = Event { user_id: "alice", value: 1 };
/// let user_b = Event { user_id: "bob", value: 10 };
/// let user_a_second = Event { user_id: "alice", value: 2 };
///
/// let mut queue = FairQueue::new();
/// queue.insert(&user_a_first);
/// queue.insert(&user_a_second);
/// queue.insert(&user_b);
///
/// assert_eq!(queue.pop(), Some(&user_a_first));
/// assert_eq!(queue.pop(), Some(&user_b));
/// assert_eq!(queue.pop(), Some(&user_a_second));
/// assert!(queue.pop().is_none());
/// ```
pub struct FairQueue<'a, V: FairGroup> {
    groups: Vec<VecDeque<&'a V>>,
    pointer: usize,
    len: usize,
}

impl<'a, V: FairGroup> FairQueue<'a, V> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            pointer: 0,
            len: 0,
        }
    }

    /// Inserts a new item into the queue, ensuring spatial distancing between items of the same group.
    pub fn insert(&mut self, value: &'a V) {
        if let Some(group) = self.groups.iter_mut().find(|group| {
            group
                .front()
                .is_some_and(|v| ptr::eq(*v, value) || (*v).is_same_group(value))
        }) {
            group.push_back(value);
        } else {
            let mut new_group = VecDeque::new();
            new_group.push_back(value);
            self.groups.push(new_group);
        }
        self.len += 1;
    }

    /// Retrieves the next item in the queue (FIFO) while maintaining spatial distancing.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<&'a V> {
        if self.len == 0 {
            return None;
        }

        loop {
            if self.groups.is_empty() {
                self.pointer = 0;
                return None;
            }

            if self.pointer >= self.groups.len() {
                self.pointer = 0;
            }

            let group = &mut self.groups[self.pointer];

            if let Some(item) = group.pop_front() {
                self.len -= 1;

                if group.is_empty() {
                    self.groups.swap_remove(self.pointer);
                    if self.groups.is_empty() || self.pointer >= self.groups.len() {
                        self.pointer = 0;
                    }
                } else if !self.groups.is_empty() {
                    self.pointer = (self.pointer + 1) % self.groups.len();
                }

                return Some(item);
            }

            self.groups.swap_remove(self.pointer);
            if self.groups.is_empty() {
                self.pointer = 0;
                return None;
            }
            if self.pointer >= self.groups.len() {
                self.pointer = 0;
            }
        }
    }

    /// Peeks at the next item in the queue without removing it.
    #[inline(always)]
    #[must_use]
    pub fn peek(&self) -> Option<&'a V> {
        if self.groups.is_empty() {
            return None;
        }

        self.groups.get(self.pointer)?.front().copied()
    }

    /// Returns the number of enqueued items.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true when the queue holds no items.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of distinct groups tracked by the queue.
    #[inline(always)]
    #[must_use]
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Iterates over the current head item of each group without consuming them.
    #[inline(always)]
    #[must_use]
    pub fn group_heads(&self) -> QueueGroupHeads<'_, 'a, V> {
        QueueGroupHeads {
            iter: self.groups.iter(),
        }
    }

    /// Collects group heads into a vector (requires the `std` feature).
    #[cfg(feature = "std")]
    #[must_use]
    pub fn group_heads_vec(&self) -> std::vec::Vec<&'a V> {
        self.group_heads().collect()
    }

    /// Clears all items and resets the round-robin pointer.
    pub fn clear(&mut self) {
        self.groups.clear();
        self.pointer = 0;
        self.len = 0;
    }
}

impl<V: FairGroup> Default for FairQueue<'_, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over the first element of each group.
pub struct QueueGroupHeads<'queue, 'value, V: FairGroup> {
    iter: slice::Iter<'queue, VecDeque<&'value V>>,
}

impl<'queue, 'value, V: FairGroup> Iterator for QueueGroupHeads<'queue, 'value, V> {
    type Item = &'value V;

    fn next(&mut self) -> Option<Self::Item> {
        for group in &mut self.iter {
            if let Some(item) = group.front() {
                return Some(*item);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, PartialEq)]
    struct Event {
        timestamp: u32,
        user_id: &'static str,
        group: usize,
    }

    impl FairGroup for Event {
        fn is_same_group(&self, other: &Self) -> bool {
            self.group == other.group
        }
    }

    #[test]
    fn test_spaced_fairness() {
        let event1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let event2 = Event {
            timestamp: 2,
            user_id: "user2",
            group: 1,
        };
        let event3 = Event {
            timestamp: 3,
            user_id: "user1",
            group: 0,
        };
        let event4 = Event {
            timestamp: 4,
            user_id: "user3",
            group: 2,
        };
        let event5 = Event {
            timestamp: 5,
            user_id: "user2",
            group: 1,
        };
        let event6 = Event {
            timestamp: 6,
            user_id: "user1",
            group: 0,
        };
        let event7 = Event {
            timestamp: 7,
            user_id: "user1",
            group: 0,
        };
        let event8 = Event {
            timestamp: 8,
            user_id: "user3",
            group: 2,
        };

        let mut queue = FairQueue::new();

        queue.insert(&event1);
        queue.insert(&event2);
        queue.insert(&event3);
        queue.insert(&event4);
        queue.insert(&event5);
        queue.insert(&event6);
        queue.insert(&event7);
        queue.insert(&event8);

        assert_eq!(queue.pop(), Some(&event1));
        assert_eq!(queue.pop(), Some(&event2));
        assert_eq!(queue.pop(), Some(&event4));
        assert_eq!(queue.pop(), Some(&event3));
        assert_eq!(queue.pop(), Some(&event5));
        assert_eq!(queue.pop(), Some(&event8));
        assert_eq!(queue.pop(), Some(&event6));
        assert_eq!(queue.pop(), Some(&event7));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_len_and_group_count() {
        let event = Event {
            timestamp: 0,
            user_id: "user1",
            group: 0,
        };
        let other = Event {
            timestamp: 1,
            user_id: "user2",
            group: 1,
        };

        let mut queue = FairQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.group_count(), 0);

        queue.insert(&event);
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.group_count(), 1);

        queue.insert(&other);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.group_count(), 2);

        assert_eq!(queue.pop(), Some(&event));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.group_count(), 1);

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.group_count(), 0);
    }

    #[test]
    fn test_peek_matches_pop() {
        let event = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let other = Event {
            timestamp: 2,
            user_id: "user2",
            group: 1,
        };

        let mut queue = FairQueue::new();
        queue.insert(&event);
        queue.insert(&other);

        assert_eq!(queue.peek(), Some(&event));
        assert_eq!(queue.peek(), Some(&event));
        assert_eq!(queue.pop(), Some(&event));
        assert_eq!(queue.peek(), Some(&other));
    }

    #[test]
    fn test_group_heads_snapshot() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let b1 = Event {
            timestamp: 2,
            user_id: "user2",
            group: 1,
        };
        let b2 = Event {
            timestamp: 3,
            user_id: "user2",
            group: 1,
        };
        let c1 = Event {
            timestamp: 4,
            user_id: "user3",
            group: 2,
        };

        let mut queue = FairQueue::new();
        queue.insert(&a1);
        queue.insert(&b1);
        queue.insert(&b2);
        queue.insert(&c1);

        let mut heads = queue.group_heads();
        assert_eq!(heads.next(), Some(&a1));
        assert_eq!(heads.next(), Some(&b1));
        assert_eq!(heads.next(), Some(&c1));
        assert_eq!(heads.next(), None);

        #[cfg(feature = "std")]
        {
            let collected = queue.group_heads_vec();
            assert_eq!(collected, vec![&a1, &b1, &c1]);
        }

        // Ensure borrowing state not consumed.
        assert_eq!(queue.peek(), Some(&a1));
    }

    #[test]
    fn test_pointer_wraps_after_group_removal() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let b1 = Event {
            timestamp: 2,
            user_id: "user2",
            group: 1,
        };
        let c1 = Event {
            timestamp: 3,
            user_id: "user3",
            group: 2,
        };
        let d1 = Event {
            timestamp: 4,
            user_id: "user4",
            group: 3,
        };

        let mut queue = FairQueue::new();
        queue.insert(&a1);
        queue.insert(&b1);
        queue.insert(&c1);

        assert_eq!(queue.pop(), Some(&a1));
        queue.insert(&d1);

        let mut groups = Vec::new();
        while let Some(item) = queue.pop() {
            groups.push(item.group);
        }

        groups.sort_unstable();
        assert_eq!(groups, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    proptest! {
        #[test]
        fn prop_queue_preserves_spacing(groups in proptest::collection::vec(0usize..4, 1..32)) {
            const IDS: [&str; 4] = ["g0", "g1", "g2", "g3"];

            let mut events = Vec::with_capacity(groups.len());
            for (idx, group) in groups.iter().enumerate() {
                events.push(Event {
                    timestamp: idx as u32,
                    user_id: IDS[*group],
                    group: *group,
                });
            }

            let mut queue = FairQueue::new();
            for event in &events {
                queue.insert(event);
            }

            let mut remaining = [0usize; 4];
            for event in &events {
                remaining[event.group] += 1;
            }

            let mut last_group: Option<usize> = None;
            while let Some(event) = queue.pop() {
                let gid = event.group;

                let other_pending = remaining
                    .iter()
                    .enumerate()
                    .any(|(idx, &count)| idx != gid && count > 0);

                if other_pending && let Some(prev) = last_group {
                    prop_assert_ne!(prev, gid);
                }

                remaining[gid] = remaining[gid].saturating_sub(1);
                last_group = Some(gid);
            }
        }
    }
}
