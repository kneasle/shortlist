#![deny(clippy::cargo)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct Shortlist<T: Ord> {
    heap: BinaryHeap<Reverse<T>>,
}

impl<T: Ord> Shortlist<T> {
    pub fn new(capacity: usize) -> Self {
        Shortlist {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }
}
