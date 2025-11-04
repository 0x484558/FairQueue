use alloc::{vec, vec::Vec};
use core::{ptr, slice};

use crate::FairGroup;

/// Spatially distancing fair stack.
/// Last in, first out inside a group while rotating across groups fairly.
///
/// ```
/// use fairqueue::{FairGroup, FairStack};
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
/// let a1 = Event { user_id: "alice", value: 1 };
/// let a2 = Event { user_id: "alice", value: 2 };
/// let b1 = Event { user_id: "bob", value: 10 };
///
/// let mut stack = FairStack::new();
/// stack.push(&a1);
/// stack.push(&a2);
/// stack.push(&b1);
///
/// assert_eq!(stack.pop(), Some(&a2));
/// assert_eq!(stack.pop(), Some(&b1));
/// assert_eq!(stack.pop(), Some(&a1));
/// assert!(stack.pop().is_none());
/// ```
pub struct FairStack<'a, V: FairGroup> {
    groups: Vec<Vec<&'a V>>,
    pointer: usize,
    len: usize,
}

impl<'a, V: FairGroup> FairStack<'a, V> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            pointer: 0,
            len: 0,
        }
    }

    /// Pushes an item while ensuring the group keeps participating in round-robin order.
    pub fn push(&mut self, value: &'a V) {
        if let Some(group) = self.groups.iter_mut().find(|group| {
            group
                .last()
                .is_some_and(|v| ptr::eq(*v, value) || (*v).is_same_group(value))
        }) {
            group.push(value);
        } else {
            self.groups.push(vec![value]);
        }
        self.len += 1;
    }

    /// Pops the next item while rotating across groups fairly.
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

            if let Some(item) = group.pop() {
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

    /// Peeks at the next item due for popping.
    #[inline(always)]
    #[must_use]
    pub fn peek(&self) -> Option<&'a V> {
        if self.groups.is_empty() {
            return None;
        }

        self.groups.get(self.pointer)?.last().copied()
    }

    /// Peeks at the next item for a given group without disturbing rotation.
    #[must_use]
    pub fn peek_group(&self, sample: &V) -> Option<&'a V> {
        self.groups
            .iter()
            .find(|group| {
                group
                    .last()
                    .is_some_and(|v| ptr::eq(*v, sample) || (*v).is_same_group(sample))
            })
            .and_then(|group| group.last().copied())
    }

    /// Returns the number of enqueued items.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true when the stack holds no items.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of distinct groups tracked by the stack.
    #[inline(always)]
    #[must_use]
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Iterates over the current top item of each group without consuming them.
    #[inline(always)]
    #[must_use]
    pub fn group_heads(&self) -> StackGroupHeads<'_, 'a, V> {
        StackGroupHeads {
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

impl<V: FairGroup> Default for FairStack<'_, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over the last element of each group.
pub struct StackGroupHeads<'stack, 'value, V: FairGroup> {
    iter: slice::Iter<'stack, Vec<&'value V>>,
}

impl<'stack, 'value, V: FairGroup> Iterator for StackGroupHeads<'stack, 'value, V> {
    type Item = &'value V;

    fn next(&mut self) -> Option<Self::Item> {
        for group in &mut self.iter {
            if let Some(item) = group.last() {
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
    fn alternates_groups() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let a2 = Event {
            timestamp: 2,
            user_id: "user1",
            group: 0,
        };
        let b1 = Event {
            timestamp: 3,
            user_id: "user2",
            group: 1,
        };
        let b2 = Event {
            timestamp: 4,
            user_id: "user2",
            group: 1,
        };

        let mut stack = FairStack::new();
        stack.push(&a1);
        stack.push(&a2);
        stack.push(&b1);
        stack.push(&b2);

        assert_eq!(stack.pop(), Some(&a2));
        assert_eq!(stack.pop(), Some(&b2));
        assert_eq!(stack.pop(), Some(&a1));
        assert_eq!(stack.pop(), Some(&b1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn len_group_count_and_peek() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let a2 = Event {
            timestamp: 2,
            user_id: "user1",
            group: 0,
        };
        let b1 = Event {
            timestamp: 3,
            user_id: "user2",
            group: 1,
        };

        let mut stack = FairStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.group_count(), 0);

        stack.push(&a1);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.group_count(), 1);
        assert_eq!(stack.peek(), Some(&a1));

        stack.push(&a2);
        stack.push(&b1);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.group_count(), 2);
        assert_eq!(stack.peek(), Some(&a2));
        assert_eq!(stack.peek_group(&a2), Some(&a2));
        assert_eq!(stack.peek_group(&b1), Some(&b1));

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.group_count(), 0);
    }

    #[test]
    fn test_group_heads_snapshot() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
            group: 0,
        };
        let a2 = Event {
            timestamp: 2,
            user_id: "user1",
            group: 0,
        };
        let b1 = Event {
            timestamp: 3,
            user_id: "user2",
            group: 1,
        };
        let c1 = Event {
            timestamp: 4,
            user_id: "user3",
            group: 2,
        };

        let mut stack = FairStack::new();
        stack.push(&a1);
        stack.push(&a2);
        stack.push(&b1);
        stack.push(&c1);

        let mut heads = stack.group_heads();
        assert_eq!(heads.next(), Some(&a2));
        assert_eq!(heads.next(), Some(&b1));
        assert_eq!(heads.next(), Some(&c1));
        assert_eq!(heads.next(), None);

        assert_eq!(stack.peek(), Some(&a2));

        #[cfg(feature = "std")]
        {
            let collected = stack.group_heads_vec();
            assert_eq!(collected, vec![&a2, &b1, &c1]);
        }
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

        let mut stack = FairStack::new();
        stack.push(&a1);
        stack.push(&b1);
        stack.push(&c1);

        assert_eq!(stack.pop(), Some(&a1));

        stack.push(&d1);
        let mut groups = Vec::new();
        while let Some(item) = stack.pop() {
            groups.push(item.group);
        }

        groups.sort_unstable();
        assert_eq!(groups, vec![1, 2, 3]);
        assert!(stack.is_empty());
    }

    proptest! {
        #[test]
        fn prop_stack_preserves_spacing(groups in proptest::collection::vec(0usize..4, 1..32)) {
            const IDS: [&str; 4] = ["g0", "g1", "g2", "g3"];

            let mut events = Vec::with_capacity(groups.len());
            for (idx, group) in groups.iter().enumerate() {
                events.push(Event {
                    timestamp: idx as u32,
                    user_id: IDS[*group],
                    group: *group,
                });
            }

            let mut stack = FairStack::new();
            for event in &events {
                stack.push(event);
            }

            let mut remaining = [0usize; 4];
            for event in &events {
                remaining[event.group] += 1;
            }

            let mut last_group: Option<usize> = None;
            while let Some(event) = stack.pop() {
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
