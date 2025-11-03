#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, collections::VecDeque};

/// Spatially distancing queue. First in, first out.
///
/// This queue type ensures fairness of dequeuing by ensuring that each group (determined by a key)
/// is placed as far apart as possible.
pub struct SpacedQueue<'a, K: Ord, V> {
    item_map: BTreeMap<&'a K, VecDeque<&'a V>>, // Stores items grouped by keys
    group_order: VecDeque<&'a K>, // Tracks the round-robin order of keys
    pointer: usize, // Optimized pointer to track the current key index
}

impl<'a, K: Ord, V> SpacedQueue<'a, K, V> {
    /// Creates a new empty SpacedQueue.
    pub fn new() -> Self {
        SpacedQueue {
            item_map: BTreeMap::new(),
            group_order: VecDeque::new(),
            pointer: 0,
        }
    }

    /// Inserts a new item into the queue, ensuring spatial distancing between items with the same key.
    pub fn insert(&mut self, key: &'a K, value: &'a V) {
        // Add the key to the round-robin order if it's new
        let entry = self.item_map.entry(key).or_insert_with(|| {
            self.group_order.push_back(key); // Push into the VecDeque
            VecDeque::new()
        });

        // Insert the item into the appropriate key's queue
        entry.push_back(value);
    }

    /// Retrieves the next item in the queue (FIFO) while maintaining spatial distancing.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<&'a V> {
        if self.group_order.is_empty() {
            return None; // No items to pop
        }

        let mut num_checked = 0;
        while num_checked < self.group_order.len() {
            let key = self.group_order[self.pointer];

            if let Some(group) = self.item_map.get_mut(key) {
                if let Some(item) = group.pop_front() {
                    // Remove the key from the group order if the queue is now empty
                    if group.is_empty() {
                        self.item_map.remove(key);
                        self.group_order.remove(self.pointer);
                        if self.pointer >= self.group_order.len() {
                            self.pointer = 0;
                        }
                    } else {
                        // Move the pointer to the next key
                        self.pointer = (self.pointer + 1) % self.group_order.len();
                    }
                    return Some(item);
                }
            }

            // Move the pointer to the next key
            self.pointer = (self.pointer + 1) % self.group_order.len();
            num_checked += 1;
        }

        None
    }

    /// Peeks at the next item in the queue without removing it.
    #[inline(always)]
    pub fn peek(&self) -> Option<&&'a V> {
        if self.group_order.is_empty() {
            return None;
        }

        let key = self.group_order[self.pointer];
        self.item_map.get(key)?.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spaced_queue() {
        let key1 = 1;
        let key2 = 2;
        let key3 = 3;
        let key4 = 4;

        let value1 = 1;
        let value2 = 2;
        let value3 = 3;
        let value4 = 4;
        let value5 = 5;
        let value6 = 6;
        let value7 = 7;
        let value8 = 8;
        let value9 = 9;
        let value10 = 10;

        let mut queue = SpacedQueue::new();

        queue.insert(&key1, &value1);
        queue.insert(&key2, &value2);
        queue.insert(&key3, &value3);
        queue.insert(&key4, &value4);
        queue.insert(&key2, &value5);
        queue.insert(&key2, &value6);
        queue.insert(&key4, &value7);
        queue.insert(&key1, &value8);
        queue.insert(&key3, &value9);
        queue.insert(&key2, &value10);

        assert_eq!(queue.pop(), Some(&value1));
        assert_eq!(queue.pop(), Some(&value2));
        assert_eq!(queue.pop(), Some(&value3));
        assert_eq!(queue.pop(), Some(&value4));
        assert_eq!(queue.pop(), Some(&value8));
        assert_eq!(queue.pop(), Some(&value5));
        assert_eq!(queue.pop(), Some(&value9));
        assert_eq!(queue.pop(), Some(&value7));
        assert_eq!(queue.pop(), Some(&value6));
        assert_eq!(queue.pop(), Some(&value10));
        assert_eq!(queue.pop(), None);
    }
}
