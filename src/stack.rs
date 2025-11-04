use alloc::{vec, vec::Vec};
use core::ptr;

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
    fn alternates_groups() {
        let a1 = Event {
            timestamp: 1,
            user_id: "user1",
        };
        let a2 = Event {
            timestamp: 2,
            user_id: "user1",
        };
        let b1 = Event {
            timestamp: 3,
            user_id: "user2",
        };
        let b2 = Event {
            timestamp: 4,
            user_id: "user2",
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
        };
        let a2 = Event {
            timestamp: 2,
            user_id: "user1",
        };
        let b1 = Event {
            timestamp: 3,
            user_id: "user2",
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
}
