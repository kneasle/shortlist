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

    pub fn from_slice(capacity: usize, contents: &[T]) -> Self
    where
        T: Clone,
    {
        let mut shortlist = Shortlist::new(capacity);
        shortlist.append_slice(contents);
        shortlist
    }

    // Tested
    pub fn from_iter(capacity: usize, contents: impl IntoIterator<Item = T>) -> Self {
        let mut shortlist = Shortlist::new(capacity);
        shortlist.append(contents);
        shortlist
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

    #[inline]
    pub fn append(&mut self, contents: impl IntoIterator<Item = T>) {
        for i in contents {
            self.push(i);
        }
    }

    // Tested
    #[inline]
    pub fn append_slice(&mut self, contents: &[T])
    where
        T: Clone,
    {
        for i in contents {
            self.push(i.clone());
        }
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        // We transmute the memory in order to convert the `Reverse<T>`s into `T`s without cloning
        // the data.  This is fine because in memory, `Reverse<T>`s are identical to `T`s, so
        // transmuting the `Vec` is completely allowed.
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
        // We transmute the memory in order to convert the `Reverse<T>`s into `T`s without cloning
        // the data.  This is fine because in memory, `Reverse<T>`s are identical to `T`s, so
        // transmuting the `Vec` is completely allowed.
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

    /* ===== HELPER FUNCTIONS ===== */

    /// Given a sorted [`Vec`] of input values and a sorted [`Vec`] of the values taken from a
    /// [`Shortlist`] of that item, checks that the [`Shortlist`] behaved correctly.
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

    /// Generates a random capacity and randomised input [`Vec`] to be used as a test sample.
    fn gen_sample_input(rng: &mut impl Rng) -> (usize, Vec<usize>) {
        // Decide how much capacity the shortlist will have
        let capacity = rng.gen_range(1, 100);
        // Make empty collections
        let mut input_values: Vec<usize> = Vec::new();
        // Populate both collections with the same values
        for _ in 0..rng.gen_range(1, 1000) {
            let val = rng.gen_range(0, 1000);
            input_values.push(val);
        }
        (capacity, input_values)
    }

    /// Generates a randomised chunk of input data and a [`Shortlist`] built from that data.  The
    /// [`Vec`] returned is always sorted, though the [`Shortlist`] is generated from the unsorted
    /// data to be a fair test.
    fn generate_input_and_shortlist(rng: &mut impl Rng) -> (Vec<usize>, Shortlist<usize>) {
        let (capacity, mut input_values) = gen_sample_input(rng);
        let shortlist: Shortlist<usize> = Shortlist::from_slice(capacity, &input_values);
        // Sort the input values and return
        input_values.sort();
        (input_values, shortlist)
    }

    /// Test a given check over [`Shortlist`]s many many times.
    fn check_correctness(check: impl Fn(Vec<usize>, Shortlist<usize>) -> ()) {
        let mut rng = thread_rng();
        // Make a shortlist with a known set of values
        for _ in 1..10_000 {
            let (input_values, shortlist) = generate_input_and_shortlist(&mut rng);
            // Check that the shortlist contains a suffix of the sorted reference vec
            check(input_values, shortlist);
        }
    }

    /* ===== TESTING FUNCTIONS ===== */

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
            let capacity = shortlist.capacity();
            let shortlist_vec = shortlist.into_sorted_vec();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_sorted_vec_safe() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let shortlist_vec = shortlist.into_sorted_vec_safe();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_vec() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_vec_safe() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let mut shortlist_vec = shortlist.into_vec_safe();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(values, borrowed_shortlist_vec, capacity);
        });
    }

    #[test]
    fn append() {
        let mut rng = thread_rng();
        // Make a shortlist with a known set of values
        for _ in 1..10_000 {
            let (capacity, mut input_values) = gen_sample_input(&mut rng);
            let shortlist: Shortlist<usize> =
                Shortlist::from_iter(capacity, input_values.iter().copied());
            // Sort the input values
            input_values.sort();
            // Check that the shortlist contains a suffix of the sorted reference vec
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(input_values, borrowed_shortlist_vec, capacity);
        }
    }

    #[test]
    fn capacity_and_len() {
        let mut rng = thread_rng();
        // Make a shortlist with a known set of values
        for _ in 1..10_000 {
            // Generate a test sample
            let (capacity, mut input_values) = gen_sample_input(&mut rng);
            // Add the values to the shortlist, asserting that the length and capacity are always
            // correct
            let mut shortlist: Shortlist<usize> = Shortlist::new(capacity);
            for (i, val) in input_values.iter().copied().enumerate() {
                // The length of the shortlist should increase every time we add an element, unless
                // the shortlist is full in which case it will stay at the capacity forever
                assert_eq!(shortlist.len(), i.min(capacity));
                // The capacity of the shortlist should never change
                assert_eq!(shortlist.capacity(), capacity);
                // Add the new value
                shortlist.push(val);
            }
            // Sort the input values
            input_values.sort();
            // Check that the shortlist contains a suffix of the sorted reference vec
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            let borrowed_shortlist_vec: Vec<&usize> = shortlist_vec.iter().collect();
            check_sorted_vecs(input_values, borrowed_shortlist_vec, capacity);
        }
    }
}
