//! A data structure to track the largest items pushed to it with no heap allocations and `O(1)`
//! amortized time per push.
//!
//! # Features
//! - Time complexity is `O(1)` per push amortized over every possible input sequence, and 
//!   `O(log n)` worst case (if the inputs are already sorted)
//! - No heap allocations except when creating a new `Shortlist`
//! - 0 dependencies, and only ~150 lines of source code
//! - 'Safe' versions are provided for functions that contain `unsafe` code in order to prevent
//!   heap allocations
//!
//! # The Problem
//! Suppose that you are running a brute force search over a very large search space, but want to
//! keep more than just the single best item - for example, you want to find the best 100 items out
//! of a search of billions options.
//!
//! I.e. you want to implement the following function:
//! ```
//! fn get_best<T: Ord>(
//!     big_computation: impl Iterator<Item = T>,
//!     n: usize
//! ) -> Vec<T> {
//!     // Somehow get the `n` largest items produced by `big_computation` ...
//!     # vec![]
//! }
//! ```
//!
//! # A bad solution
//! The naive approach to this would be to store every item that we searched.  Then once the search
//! is complete, sort this list and then take however many items we need from the end of the list.
//! This corresponds to roughly the following code:
//! ```
//! fn get_best<T: Ord>(
//!     big_computation: impl Iterator<Item = T>,
//!     n: usize
//! ) -> Vec<T> {
//!     // Collect all the results into a big sorted vec
//!     let mut giant_vec: Vec<T> = big_computation.collect();
//!     giant_vec.sort();
//!     // Return the last and therefore biggest n items with some iterator magic
//!     giant_vec.drain(..).rev().take(n).rev().collect()
//! }
//!
//! # // Check that this does in fact do the right thing, albeit very slowly
//! # assert_eq!(
//! #     get_best([0, 3, 2, 1, 4, 5].iter().copied(), 3),
//! #     vec![3, 4, 5]
//! # );
//! ```
//!
//! But this is massively inefficient in two ways:
//! - Sorting very large lists is very slow, and we are sorting potentially billions of items that
//!   we will never need.
//! - For any decently large search space, storing these items will likely crash the computer by
//!   making it run out of memory.
//! 
//! # The solution used by this crate
//! This is where using a `Shortlist` is useful.
//!
//! A `Shortlist` is a datastructure that will dynamically keep a 'shortlist' of the best items
//! given to it so far, with `O(1)` amortized time for pushing new items.  It will also only perform
//! one heap allocation when the `Shortlist` is created and every subsequent operation will be
//! allocation free.  Therefore, to the user of this library the code becomes:
//! ```
//! use shortlist::Shortlist;
//!
//! fn get_best<T: Ord>(
//!     big_computation: impl Iterator<Item = T>,
//!     n: usize
//! ) -> Vec<T> {
//!     // Create a new Shortlist that will take at most `n` items
//!     let mut shortlist = Shortlist::new(n);
//!     // Feed it all the results from `big_computation`
//!     for v in big_computation {
//!         shortlist.push(v);
//!     }
//!     // Return the shortlisted values as a sorted vec
//!     shortlist.into_sorted_vec()
//! }
//!
//! # // Check that this does in fact do the right thing
//! # assert_eq!(
//! #     get_best([0, 3, 2, 1, 4, 5].iter().copied(), 3),
//! #     vec![3, 4, 5]
//! # );
//! ```
//!
//! Or as a one-liner:
//! ```
//! use shortlist::Shortlist;
//!
//! fn get_best<T: Ord>(big_computation: impl Iterator<Item = T>, n: usize) -> Vec<T> {
//!     Shortlist::from_iter(n, big_computation).into_sorted_vec()
//! }
//!
//! # // Check that this does in fact do the right thing
//! # assert_eq!(
//! #     get_best([0, 3, 2, 1, 4, 5].iter().copied(), 3),
//! #     vec![3, 4, 5]
//! # );
//! ```
//!
//! In both cases, the code will make exactly one heap allocation (to reserve space for the
//! `Shortlist`).

#![deny(clippy::cargo)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A datastructure that tracks the largest items pushed to it with no heap allocations and `O(1)`
/// amortized time per push.
///
/// A `Shortlist` is a datastructure that will dynamically keep a 'shortlist' of the best items
/// given to it so far, with `O(1)` amortized time for pushing new items.  It will also only perform
/// one heap allocation when the `Shortlist` is created and every subsequent operation will be
/// allocation free.
///
/// # Example
/// Find the top `100` values from 1000 randomly generated integers without storing more than 100
/// integers on the heap at a time.
/// ```
/// use shortlist::Shortlist;
/// use rand::prelude::*;
///
/// // Make a Shortlist and tell it to allocate space for 100 usizes
/// let mut shortlist: Shortlist<usize> = Shortlist::new(100);
/// // Push 1000 random values between 0 and 10,000
/// let mut rng = thread_rng();
/// for _ in 0..1000 {
///     shortlist.push(rng.gen_range(0, 10_000));
/// }
/// // Consume the shortlist and print its top 100 items in ascending order
/// println!("{:?}", shortlist.into_sorted_vec());
/// ```
#[derive(Debug, Clone)]
pub struct Shortlist<T> {
    heap: BinaryHeap<Reverse<T>>,
}

impl<T: Ord> Shortlist<T> {
    /// Creates a new empty `Shortlist` with a given capacity.
    ///
    /// The capacity is the maximum number of items that the `Shortlist` will store at an any one
    /// time.
    /// Creating a new `Shortlist` causes one heap allocation, but will allocate enough memory
    /// to make sure that all subsequent operations cause no heap allocations.
    ///
    /// # Panics
    /// Creating a `Shortlist` with capacity is a logical error and will cause a panic.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let shortlist: Shortlist<u64> = Shortlist::new(42);
    /// assert_eq!(shortlist.capacity(), 42);
    /// assert!(shortlist.is_empty());
    /// ```
    pub fn new(capacity: usize) -> Shortlist<T> {
        assert!(capacity > 0, "Cannot create a Shortlist with capacity 0.");
        Shortlist {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    /// Creates a new `Shortlist` with a given capacity that contains [`Clone`]s of the largest
    /// items of a given slice.
    ///
    /// As with [`Shortlist::new`], this performs one heap allocation but every further operation
    /// on the `Shortlist` will not.
    ///
    /// If you want to `move` rather than `clone` the data, consider using [`Shortlist::from_iter`]
    /// instead.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// assert_eq!(shortlist.into_sorted_vec(), vec![5, 6, 6, 7]);
    /// ```
    pub fn from_slice(capacity: usize, contents: &[T]) -> Shortlist<T>
    where
        T: Clone,
    {
        let mut shortlist = Shortlist::new(capacity);
        shortlist.append_slice(contents);
        shortlist
    }

    /// Creates a new `Shortlist` with a given capacity that contains the largest items consumed
    /// from a given collection.
    ///
    /// As with [`Shortlist::new`], this performs one heap allocation but every further operation
    /// on the `Shortlist` will not.
    ///
    /// This does not [`Clone`] the items but instead consumes the [`Iterator`] by either moving
    /// all the values into the [`Shortlist`] or dropping them.
    /// If you would rather [`Clone`] the contents of the collection (so that the collection does
    /// not have to be consumed), consider using [`Shortlist::from_slice`] or using the
    /// [`cloned`](Iterator::cloned) iterator extension.
    ///
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_iter(4, contents.iter().copied());
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// assert_eq!(shortlist.into_vec(), vec![5, 6, 6, 7]);
    /// ```
    pub fn from_iter(capacity: usize, contents: impl IntoIterator<Item = T>) -> Shortlist<T> {
        let mut shortlist = Shortlist::new(capacity);
        shortlist.append(contents);
        shortlist
    }

    /// Add an item to the `Shortlist`.
    ///
    /// Because capacity of a `Shortlist` is fixed, once this capacity is reached any new items
    /// will either be immediately dropped (if it is not large enough to make the shortlist) or the
    /// new item will cause an existing item in the `Shortlist` to be dropped.
    ///
    /// If the `item` is big enough and there are at least two minimum values, exactly which of
    /// these minimum items will be dropped is an implementation detail of the underlying
    /// [`BinaryHeap`] and cannot be relied upon.
    ///
    /// # Time Complexity
    /// The amortized cost of this operation, over all possible input sequence is `O(1)` (same as
    /// [`BinaryHeap::push`]).
    /// This degrades the more sorted the input sequence is.
    /// However, **unlike** [`BinaryHeap::push`] this will never reallocate, so the worst case cost of
    /// any single `push` is `O(log n)` where `n` is the length of the `Shortlist`.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // Keep track of the 3 largest items so far.
    /// let mut shortlist = Shortlist::new(3);
    ///
    /// // The first two values will get added regardless of how small they are
    /// shortlist.push(0);
    /// shortlist.push(0);
    /// assert_eq!(shortlist.len(), 2);
    /// // Adding two more values will cause one of the 0s to get dropped from the Shortlist.
    /// // However, we don't know which `0` is still in the Shortlist
    /// shortlist.push(3);
    /// shortlist.push(4);
    /// // We now expect the shortlist to contain [0, 3, 4]
    /// assert_eq!(shortlist.into_sorted_vec(), vec![0, 3, 4]);
    /// ```
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

    /// Add an item to the `Shortlist` by reference, cloning it only if necessary.
    ///
    /// This is almost identical to [`Shortlist::push`], but gives better performance when cloning
    /// items since this will only [`Clone`] that item when it is added to the `Shortlist`.
    ///
    /// # Time Complexity
    /// Same as [`Shortlist::push`].
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // Keep track of the 3 largest items so far.
    /// let mut shortlist: Shortlist<String> = Shortlist::new(3);
    ///
    /// // The first 3 strings will be added and therefore cloned
    /// shortlist.clone_push(&"Aardvark".to_string());
    /// shortlist.clone_push(&"Zebra".to_string());
    /// shortlist.clone_push(&"Manatee".to_string());
    /// assert_eq!(
    ///     shortlist.sorted_cloned_vec(),
    ///     vec!["Aardvark".to_string(), "Manatee".to_string(), "Zebra".to_string()]
    /// );
    /// // This will be cloned and added, causing "Aardvark" to be dropped
    /// shortlist.clone_push(&"Salamander".to_string());
    /// assert_eq!(
    ///     shortlist.sorted_cloned_vec(),
    ///     vec!["Manatee".to_string(), "Salamander".to_string(), "Zebra".to_string()]
    /// );
    /// // This won't be added but it also won't be cloned
    /// shortlist.clone_push(&"Elephant".to_string());
    /// ```
    pub fn clone_push(&mut self, item: &T)
    where
        T: Clone,
    {
        if self.heap.len() < self.heap.capacity() {
            // If the heap hasn't reached capacity we should always add the new item
            self.heap.push(Reverse(item.clone()));
        } else {
            // If the heap is non-empty and `item` is less than this minimum we should early return
            // without modifying the shortlist or cloning the item
            if let Some(current_min) = self.heap.peek() {
                if item <= &current_min.0 {
                    return;
                }
            }
            // Since the heap is at capacity and `item` is bigger than the current table minimum,
            // we have to remove the minimum value to make space for `item`
            let popped = self.heap.pop();
            debug_assert!(popped.is_some());
            self.heap.push(Reverse(item.clone()));
        }
    }

    /// Consume items from an iterator and add these to the `Shortlist`.
    ///
    /// This is equivalent to calling [`Shortlist::push`] on every item from `contents`.
    /// Similarly to [`Shortlist::from_iter`] this moves all the items rather than cloning them.
    /// If you would rather [`Clone`] the contents of the collection (so that the collection does
    /// not have to be consumed), consider using [`Shortlist::append_slice`] or using the
    /// [`cloned`](Iterator::cloned) iterator extension.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // Keep track of the 3 biggest values seen so far
    /// let mut shortlist: Shortlist<usize> = Shortlist::new(3);
    /// // After adding [0, 4, 3, 2, 5], the 3 biggest values will be [3, 4, 5]
    /// shortlist.append([0, 4, 3, 2, 5].iter().copied());
    /// assert_eq!(shortlist.sorted_cloned_vec(), vec![3, 4, 5]);
    /// // Most of these values are too small, but the 5 will cause the 3 to be
    /// // dropped from the Shortlist
    /// shortlist.append([0, 2, 2, 1, 5, 2].iter().copied());
    /// assert_eq!(shortlist.sorted_cloned_vec(), vec![4, 5, 5]);
    /// ```
    #[inline]
    pub fn append(&mut self, contents: impl IntoIterator<Item = T>) {
        for i in contents {
            self.push(i);
        }
    }

    /// Clone all items from a slice and add them to the `Shortlist`.
    ///
    /// This is _equivalent_ to calling [`Shortlist::push`] on the [`Clone`] of every
    /// item in the slice.
    /// It will, however, be faster than using [`Shortlist::push`] because it internally uses
    /// [`Shortlist::clone_push`], which only clones the values if they are added to the
    /// `Shortlist`.
    /// If you want to move the items and consume the slice rather than cloning them, consider using
    /// [`Shortlist::append`] instead.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // Keep track of the 3 biggest values seen so far
    /// let mut shortlist: Shortlist<usize> = Shortlist::new(3);
    /// // After adding [0, 4, 3, 2, 5], the 3 biggest values will be [3, 4, 5]
    /// shortlist.append_slice(&[0, 4, 3, 2, 5]);
    /// assert_eq!(shortlist.sorted_cloned_vec(), vec![3, 4, 5]);
    /// // Most of these values are too small, but the 5 will cause the 3 to be
    /// // dropped from the Shortlist
    /// shortlist.append_slice(&[0, 2, 2, 1, 5, 2]);
    /// assert_eq!(shortlist.sorted_cloned_vec(), vec![4, 5, 5]);
    /// ```
    #[inline]
    pub fn append_slice(&mut self, contents: &[T])
    where
        T: Clone,
    {
        for i in contents {
            self.clone_push(i);
        }
    }

    /// Consumes this `Shortlist` and return a [`Vec`] containing the contents of the `Shortlist` in
    /// ascending order.
    ///
    /// # Safety
    /// This uses one line of `unsafe` code to avoid allocating heap memory.
    /// It makes no assumptions about the consumer's code and has been pretty extensively
    /// tested, but if you still want to trade off the performance penalty to avoid using any
    /// `unsafe` code, use [`Shortlist::into_sorted_vec_safe`] instead.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// assert_eq!(shortlist.into_sorted_vec(), vec![5, 6, 6, 7]);
    /// ```
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

    /// Consumes this `Shortlist` and return a [`Vec`] containing the contents of the `Shortlist`
    /// in ascending order.
    ///
    /// This is an otherwise-identical version of [`into_sorted_vec`](Shortlist::into_sorted_vec)
    /// that has no `unsafe` code at the cost of having to allocate heap memory.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// assert_eq!(shortlist.into_sorted_vec_safe(), vec![5, 6, 6, 7]);
    /// ```
    pub fn into_sorted_vec_safe(self) -> Vec<T> {
        let mut reversed_vec = self.heap.into_sorted_vec();
        // Correct for the fact that the min-heap is actually a max-heap with the 'Ord' operations
        // reversed.
        reversed_vec.reverse();
        let mut vec = Vec::with_capacity(reversed_vec.len());
        for i in reversed_vec.drain(..) {
            vec.push(i.0);
        }
        vec
    }

    /// Returns a [`Vec`] containing the [`Clone`]d contents of this `Shortlist` in ascending
    /// order, without the `Shortlist` being consumed.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// assert_eq!(shortlist.sorted_cloned_vec(), vec![5, 6, 6, 7]);
    /// // Assert that the shortlist has not been consumed
    /// assert_eq!(shortlist.len(), 4);
    /// ```
    pub fn sorted_cloned_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        // We transmute the memory in order to convert the `Reverse<T>`s into `T`s without cloning
        // the data.  This is fine because in memory, `Reverse<T>`s are identical to `T`s, so
        // transmuting the `Vec` is completely allowed.
        let mut vec: Vec<T> = unsafe { std::mem::transmute(self.heap.clone().into_sorted_vec()) };
        // Correct for the fact that the min-heap is actually a max-heap with the 'Ord' operations
        // reversed.
        vec.reverse();
        vec
    }
}

impl<T> Shortlist<T> {
    /// Returns an [`Iterator`] that borrows the items in a `Shortlist`, in an arbitrary order.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let mut shortlist = Shortlist::from_slice(3, &contents);
    /// // The top 3 items of `contents` is [6, 6, 7]
    /// let mut top_3: Vec<&usize> = shortlist.iter().collect();
    /// top_3.sort();
    /// assert_eq!(top_3, vec![&6, &6, &7]);
    /// // But we can still keep using the Shortlist
    /// shortlist.push(3);
    /// ```
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.heap.iter().map(|x| &x.0)
    }

    /// Returns the maximum number of values that this `Shortlist` will store.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // Make a new Shortlist with capacity 100
    /// let shortlist: Shortlist<String> = Shortlist::new(100);
    /// assert_eq!(shortlist.capacity(), 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    /// Consumes this `Shortlist` and return a [`Vec`] containing the contents of the `Shortlist`
    /// in an arbitrary order.
    ///
    /// # Safety
    /// This uses one line of `unsafe` code to avoid allocating heap memory.
    /// It makes no assumptions about the consumer's code and has been pretty extensively
    /// tested, but if you still want to trade off the performance penalty to avoid using any
    /// `unsafe` code, use [`Shortlist::into_vec_safe`] instead.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// let mut top_4 = shortlist.into_vec();
    /// top_4.sort();
    /// assert_eq!(top_4, vec![5, 6, 6, 7]);
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        // We transmute the memory in order to convert the `Reverse<T>`s into `T`s without cloning
        // the data.  This is fine because in memory, `Reverse<T>`s are identical to `T`s, so
        // transmuting the `Vec` is completely allowed.
        unsafe { std::mem::transmute(self.heap.into_vec()) }
    }

    /// Consumes this `Shortlist` and return a [`Vec`] containing the contents of the `Shortlist`
    /// in an arbitrary order.
    ///
    /// This is an otherwise-identical version of [`into_vec`](Shortlist::into_vec) that has no
    /// `unsafe` code at the cost of having to allocate heap memory.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let contents = [0, 3, 6, 5, 2, 1, 4, 6, 7];
    /// let shortlist = Shortlist::from_slice(4, &contents);
    /// // The top 4 items of `contents` is [5, 6, 6, 7]
    /// let mut top_4 = shortlist.into_vec_safe();
    /// top_4.sort();
    /// assert_eq!(top_4, vec![5, 6, 6, 7]);
    /// ```
    pub fn into_vec_safe(self) -> Vec<T> {
        let mut reversed_vec = self.heap.into_vec();
        // move all the values out of the `Reverse`s into a different vector, and return that
        let mut vec = Vec::with_capacity(reversed_vec.len());
        for i in reversed_vec.drain(..) {
            vec.push(i.0);
        }
        vec
    }

    /// Returns the number of items in a `Shortlist`.
    ///
    /// This will never be greater than the [`capacity`](Shortlist::capacity).
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// // The shortlist starts with no items
    /// let mut shortlist = Shortlist::new(3);
    /// assert_eq!(shortlist.len(), 0);
    /// // The first 3 items will all get added, and so cause len to increase
    /// shortlist.push(4);
    /// assert_eq!(shortlist.len(), 1);
    /// shortlist.push(2);
    /// assert_eq!(shortlist.len(), 2);
    /// shortlist.push(5);
    /// assert_eq!(shortlist.len(), 3);
    /// // Adding a 4th item will cause an item to be dropped and the len to stay at 3
    /// shortlist.push(6);
    /// assert_eq!(shortlist.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns `true` if a `Shortlist` contains no items.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let mut shortlist = Shortlist::new(3);
    /// // The shortlist starts empty
    /// assert!(shortlist.is_empty());
    /// // The shortlist is not empty if we push some values
    /// shortlist.push(4);
    /// assert!(!shortlist.is_empty());
    /// shortlist.append_slice(&[0, 1, 2, 3]);
    /// assert!(!shortlist.is_empty());
    /// // If we clear the shortlist, it becomes empty
    /// shortlist.clear();
    /// assert!(shortlist.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns an [`Iterator`] that pops the items from a `Shortlist` in an arbitrary order.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let mut shortlist = Shortlist::new(3);
    /// shortlist.append_slice(&[0, 1, 5, 2, 3, 5]);
    /// // Drain the shortlist into a vector, showing that the Shortlist is then empty
    /// let mut drained_values: Vec<usize> = shortlist.drain().collect();
    /// assert!(shortlist.is_empty());
    /// // Check that we drained the right values ([3, 5, 5])
    /// drained_values.sort(); // The values are in arbitrary order
    /// assert_eq!(drained_values, vec![3, 5, 5]);
    /// ```
    #[inline]
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = T> + 'a {
        self.heap.drain().map(|x| x.0)
    }

    /// Remove and drop all the items in a `Shortlist`, leaving it empty.
    ///
    /// # Example
    /// ```
    /// use shortlist::Shortlist;
    ///
    /// let mut shortlist = Shortlist::from_slice(3, &[0, 1, 2, 3]);
    /// // If we clear the shortlist, it becomes empty
    /// assert!(!shortlist.is_empty());
    /// shortlist.clear();
    /// assert!(shortlist.is_empty());
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
    /// [`Shortlist`] of those items, checks that the [`Shortlist`] behaved correctly.
    fn check_sorted_vecs<T: Ord + Eq + std::fmt::Debug>(
        sorted_input_values: Vec<T>,
        shortlist_vec: Vec<T>,
        capacity: usize,
    ) {
        let mut debug_lines = Vec::with_capacity(1000);
        debug_lines.push("".to_string());
        debug_lines.push(format!("Input length      : {}", sorted_input_values.len()));
        debug_lines.push(format!("Shortlist capacity: {}", capacity));
        debug_lines.push(format!("Shortlist length  : {}", shortlist_vec.len()));
        // let shortlist_vec = shortlist.into_sorted_vec();
        // Check that the shortlist's length is the minimum of its capacity and the number of input
        // values
        if shortlist_vec.len() != capacity.min(sorted_input_values.len()) {
            debug_lines.push(format!("Input values: {:?}", sorted_input_values));
            debug_lines.push(format!("Shortlisted values: {:?}", shortlist_vec));
            // Print the debug info before panicking
            for line in debug_lines {
                println!("{}", line);
            }
            panic!();
        }
        // Check that `shortlist.into_sorted_vec()` produces a suffix of `input_values` (we can
        // guaruntee that the input values are sorted).
        for (val, exp_val) in shortlist_vec
            .iter()
            .rev()
            .zip(sorted_input_values.iter().rev())
        {
            if val == exp_val {
                debug_lines.push(format!("{:?} == {:?}", val, exp_val));
            } else {
                debug_lines.push(format!("{:?} != {:?}", val, exp_val));
                // Print the debug info before panicking
                for line in debug_lines {
                    println!("{}", line);
                }
                panic!();
            }
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
            let mut shortlist_vec: Vec<usize> = shortlist.iter().copied().collect();
            shortlist_vec.sort();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_sorted_vec() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let shortlist_vec = shortlist.into_sorted_vec();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_sorted_vec_safe() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let shortlist_vec = shortlist.into_sorted_vec_safe();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn sorted_cloned_vec() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let shortlist_vec = shortlist.sorted_cloned_vec();
            check_sorted_vecs(values, shortlist_vec, capacity);
            // Check that the shortlist still has its values
        });
    }

    #[test]
    fn into_vec() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn into_vec_safe() {
        check_correctness(|values, shortlist| {
            let capacity = shortlist.capacity();
            let mut shortlist_vec = shortlist.into_vec_safe();
            shortlist_vec.sort();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn drain() {
        check_correctness(|values, mut shortlist| {
            let capacity = shortlist.capacity();
            let mut shortlist_vec: Vec<usize> = shortlist.drain().collect();
            // If we have drained the shortlist, it must be empty
            assert!(shortlist.is_empty());
            // Test that drain returned the right values
            shortlist_vec.sort();
            check_sorted_vecs(values, shortlist_vec, capacity);
        });
    }

    #[test]
    fn clear() {
        check_correctness(|_values, mut shortlist| {
            // Clear the shortlist and assert that it is now empty
            shortlist.clear();
            assert!(shortlist.is_empty());
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
            check_sorted_vecs(input_values, shortlist_vec, capacity);
        }
    }

    /// Tests [`Shortlist::len`], [`Shortlist::capacity`], [`Shortlist::is_empty`]
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
                // If we have pushed any values, the shortlist cannot be empty
                assert!(!shortlist.is_empty());
            }
            // Sort the input values
            input_values.sort();
            // Check that the shortlist contains a suffix of the sorted reference vec
            let mut shortlist_vec = shortlist.into_vec();
            shortlist_vec.sort();
            check_sorted_vecs(input_values, shortlist_vec, capacity);
        }
    }
}
