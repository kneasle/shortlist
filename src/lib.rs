#![deny(clippy::cargo)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct Shortlist<T> {
    heap: BinaryHeap<Reverse<T>>,
}

impl<T: Ord> Shortlist<T> {
    pub fn new(capacity: usize) -> Self {
        Shortlist {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        if let Some(current_min) = self.heap.peek() {
            if item <= current_min.0 {
                // We early return if the min of the hashtable exists, and `item` is less than it
                // (in this case `item` does not make the shortlist.
                return;
            }
        }
        // If the heap is at capacity, we need to remove the min value from the heap, which is
        // knocked out of the shortlist by `item`.
        if self.heap.len() == self.heap.capacity() {
            self.heap.pop();
        }
        // Push the new item
        self.heap.push(Reverse(item));
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        let mut vec: Vec<T> = unsafe { std::mem::transmute(self.heap.into_sorted_vec()) };
        // Correct for the fact that the min-heap is actually a max-heap with the 'Ord' operations
        // reversed.
        vec.reverse();
        vec
    }

    #[inline]
    pub fn append(&mut self, other: &mut Shortlist<T>) {
        self.heap.append(&mut other.heap);
    }
}

impl<T> Shortlist<T> {
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.heap.iter().map(|x| &x.0)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe { std::mem::transmute(self.heap.into_vec()) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    #[inline]
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = T> + 'a {
        self.heap.drain().map(|x| x.0)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}
