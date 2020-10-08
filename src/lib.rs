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
        if self.heap.len() < self.heap.capacity() {
            // If the heap hasn't reached capacity we should always add the new item
            self.heap.push(Reverse(item));
        } else {
            // If the heap is non-empty and `item` is less than this minimum we should early return
            // without modifying the shortlist
            if let Some(current_min) = self.heap.peek() {
                if item <= current_min.0 {
                    return;
                }
            }
            // Since the heap is at capacity and `item` is bigger than the current table minimum,
            // we have to remove the minimum value to make space for `item`
            let popped = self.heap.pop();
            debug_assert!(popped.is_some());
            self.heap.push(Reverse(item));
        }
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        // We transmute the memory in order to convert the `Reverse<T>`s into `T`s
        let mut vec: Vec<T> = unsafe { std::mem::transmute(self.heap.into_sorted_vec()) };
        // Correct for the fact that the min-heap is actually a max-heap with the 'Ord' operations
        // reversed.
        vec.reverse();
        vec
    }

    pub fn into_sorted_vec_safe(self) -> Vec<T>
    where
        T: Clone,
    {
        self.heap
            .into_sorted_vec()
            .iter()
            .rev()
            .map(|x| x.0.clone())
            .collect()
    }

    #[inline]
    pub fn append(&mut self, other: &mut Shortlist<T>) {
        for i in other.drain() {
            self.push(i);
        }
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

    pub fn into_vec_safe(self) -> Vec<T>
    where
        T: Clone,
    {
        self.heap.into_vec().iter().map(|x| x.0.clone()).collect()
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

#[cfg(test)]
mod tests {
    use super::Shortlist;
    use rand::prelude::*;

    fn check_sorted_vecs<T: Ord + Eq + std::fmt::Debug>(
        sorted_input_values: Vec<T>,
        shortlist_vec: Vec<&T>,
        capacity: usize,
    ) {
        println!("");
        println!("Input length      : {}", sorted_input_values.len());
        println!("Shortlist capacity: {}", capacity);
        println!("Shortlist length  : {}", shortlist_vec.len());
        // let shortlist_vec = shortlist.into_sorted_vec();
        // Check that the shortlist's length is the minimum of its capacity and the number of input
        // values
        if shortlist_vec.len() != capacity.min(sorted_input_values.len()) {
            println!("Input values: {:?}", sorted_input_values);
            println!("Shortlisted values: {:?}", shortlist_vec);
            panic!();
        }
        // Check that `shortlist.into_sorted_vec()` produces a suffix of `input_values` (we can
        // guaruntee that the input values are sorted).
        for (val, exp_val) in shortlist_vec
            .iter()
            .rev()
            .zip(sorted_input_values.iter().rev())
        {
            println!("{:?} {:?}", val, exp_val);
            assert_eq!(val, &exp_val);
        }
    }

    fn generate_input_and_shortlist(rng: &mut impl Rng) -> (Vec<usize>, Shortlist<usize>) {
        // Decide how much capacity the shortlist will have
        let capacity = rng.gen_range(1, 100);
        // Make empty collections
        let mut input_values: Vec<usize> = Vec::new();
        let mut shortlist: Shortlist<usize> = Shortlist::new(capacity);
        // Populate both collections with the same values
        for _ in 0..rng.gen_range(1, 1000) {
            let val = rng.gen_range(0, 1000);
            input_values.push(val);
            shortlist.push(val);
        }
        // Sort the input values and return
        input_values.sort();
        (input_values, shortlist)
    }

    fn check_correctness(check: impl Fn(Vec<usize>, Shortlist<usize>) -> ()) {
        let mut rng = thread_rng();
        // Make a shortlist with a known set of values
        for _ in 1..10_000 {
            let (input_values, shortlist) = generate_input_and_shortlist(&mut rng);
            // Check that the shortlist contains a suffix of the sorted reference vec
            check(input_values, shortlist);
        }
    }

    #[test]
    fn basic_operations() {
        // Make a Shortlist and push a whole load of items onto it
        let mut shortlist: Shortlist<usize> = Shortlist::new(3);
        for i in &[4, 8, 2, 7, 5, 5, 1, 2, 9, 8] {
            shortlist.push(*i);
        }
        // Copy the items out of the Shortlist using iter, sort them and check that the correct top
        // 3 items have been returned
        let mut best_3: Vec<usize> = shortlist.iter().copied().collect();
        best_3.sort();
        assert_eq!(best_3, vec![8, 8, 9]);
    }

    #[test]
    fn iter() {
        check_correctness(|values, shortlist| {
            // Store the capacity for both tests to use
            let capacity = shortlist.capacity();
            // Unload the Shortlist using `Shortlist::iter`
            let mut shortlist_vec: Vec<&usize> = shortlist.iter().collect();
            shortlist_vec.sort();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_sorted_vec() {
        check_correctness(|values, shortlist| {
            // Store the capacity for both tests to use
            let capacity = shortlist.capacity();
            // Unload the Shortlist using `Shortlist::into_sorted_vec`
            let shortlist_vec = shortlist.into_sorted_vec();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_sorted_vec_safe() {
        check_correctness(|values, shortlist| {
            // Store the capacity for both tests to use
            let capacity = shortlist.capacity();
            // Unload the Shortlist using `Shortlist::into_sorted_vec`
            let shortlist_vec = shortlist.into_sorted_vec_safe();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_vec() {
        check_correctness(|values, shortlist| {
            // Store the capacity for both tests to use
            let capacity = shortlist.capacity();
            // Unload the Shortlist using `Shortlist::into_sorted_vec`
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_vec_safe() {
        check_correctness(|values, shortlist| {
            // Store the capacity for both tests to use
            let capacity = shortlist.capacity();
            // Unload the Shortlist using `Shortlist::into_sorted_vec`
            let mut shortlist_vec = shortlist.into_vec_safe();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }
}
