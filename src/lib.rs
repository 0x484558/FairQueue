#![no_std]

extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};
use core::{default::Default, ptr};

/// Trait for defining grouping logic for fair queuing.
/// Groups are determined based on `is_same_group`.
pub trait FairGroup {
    fn is_same_group(&self, other: &Self) -> bool;
}

/// Spatially distancing fair queue. First in, first out, ensuring that
/// each group of similar values is placed as far apart as possible.
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
/// let user_a_first = Event {
///     user_id: "alice",
///     value: 1,
/// };
/// let user_b = Event {
///     user_id: "bob",
///     value: 10,
/// };
/// let user_a_second = Event {
///     user_id: "alice",
///     value: 2,
/// };
///
/// let mut queue = FairQueue::new();
/// queue.insert(&user_a_first);
/// queue.insert(&user_a_second);
/// queue.insert(&user_b);
///
/// assert_eq!(queue.len(), 3);
/// assert_eq!(queue.group_count(), 2);
/// assert_eq!(queue.pop(), Some(&user_a_first));
/// assert_eq!(queue.peek(), Some(&user_b));
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
    pub fn peek(&self) -> Option<&'a V> {
        if self.groups.is_empty() {
            return None;
        }

        self.groups.get(self.pointer)?.front().copied()
    }

    /// Returns the number of enqueued items.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true when the queue holds no items.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of distinct groups tracked by the queue.
    #[inline(always)]
    pub fn group_count(&self) -> usize {
        self.groups.len()
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_spaced_fairness() {
        let event1 = Event {
            timestamp: 1,
            user_id: "user1",
        };
        let event2 = Event {
            timestamp: 2,
            user_id: "user2",
        };
        let event3 = Event {
            timestamp: 3,
            user_id: "user1",
        };
        let event4 = Event {
            timestamp: 4,
            user_id: "user3",
        };
        let event5 = Event {
            timestamp: 5,
            user_id: "user2",
        };
        let event6 = Event {
            timestamp: 6,
            user_id: "user1",
        };
        let event7 = Event {
            timestamp: 7,
            user_id: "user1",
        };
        let event8 = Event {
            timestamp: 8,
            user_id: "user3",
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
        };
        let other = Event {
            timestamp: 1,
            user_id: "user2",
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
        };
        let other = Event {
            timestamp: 2,
            user_id: "user2",
        };

        let mut queue = FairQueue::new();
        queue.insert(&event);
        queue.insert(&other);

        assert_eq!(queue.peek(), Some(&event));
        assert_eq!(queue.peek(), Some(&event));
        assert_eq!(queue.pop(), Some(&event));
        assert_eq!(queue.peek(), Some(&other));
    }
}
