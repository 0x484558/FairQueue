#![no_std]

extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};
use core::default::Default;

/// Trait for defining grouping logic for fair queuing.
/// Groups are determined based on `is_same_group`.
pub trait FairGroup {
    fn is_same_group(&self, other: &Self) -> bool;
}

/// Spatially distancing fair queue. First in, first out, ensuring that
/// each group of similar values is placed as far apart as possible.
pub struct FairQueue<'a, V: FairGroup> {
    groups: Vec<VecDeque<&'a V>>,
    pointer: usize,
}

impl<'a, V: FairGroup> FairQueue<'a, V> {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            pointer: 0,
        }
    }

    /// Inserts a new item into the queue, ensuring spatial distancing between items of the same group.
    pub fn insert(&mut self, value: &'a V) {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|group| group.front().map_or(false, |v| v.is_same_group(value)))
        {
            group.push_back(value);
        } else {
            let mut new_group = VecDeque::new();
            new_group.push_back(value);
            self.groups.push(new_group);
        }
    }

    /// Retrieves the next item in the queue (FIFO) while maintaining spatial distancing.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<&'a V> {
        for _ in 0..self.groups.len() {
            let pointer = self.pointer;
            // Optimistically move queue pointer to the next group
            self.pointer = (pointer + 1) % self.groups.len();

            let group = &mut self.groups[pointer];
            let item = group.pop_front();

            if item.is_some() {
                if group.is_empty() {
                    self.groups.remove(pointer);
                    if pointer < self.groups.len() {
                        self.pointer = pointer;
                    } else {
                        self.pointer = 0;
                    }
                }
                return item;
            }
        }

        None
    }

    /// Peeks at the next item in the queue without removing it.
    #[inline(always)]
    pub fn peek(&self) -> Option<&&'a V> {
        if self.groups.is_empty() {
            return None;
        }

        self.groups.get(self.pointer)?.front()
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
}
