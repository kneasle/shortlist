# shortlist

A data structure to track the largest items pushed to it with no heap allocations and `O(1)`
amortized time per push.

## Features
- Time complexity is `O(1)` per push amortized over every possible input sequence, and
  `O(log n)` worst case (if the inputs are already sorted)
- No heap allocations except when creating a new `Shortlist`
- 0 dependencies, and only ~150 lines of source code
- 'Safe' versions are provided for functions that contain `unsafe` code in order to prevent
  heap allocations

## The Problem
Suppose that you are running a brute force search over a very large search space, but want to
keep more than just the single best item - for example, you want to find the best 100 items out
of a search of billions options.

I.e. you want to implement the following function:
```rust
fn get_best<T: Ord>(
    big_computation: impl Iterator<Item = T>,
    n: usize
) -> Vec<T> {
    // Somehow get the `n` largest items produced by `big_computation` ...
    # vec![]
}
```

## A bad solution
The naive approach to this would be to store every item that we searched.  Then once the search
is complete, sort this list and then take however many items we need from the end of the list.
This corresponds to roughly the following code:
```rust
fn get_best<T: Ord>(
    big_computation: impl Iterator<Item = T>,
    n: usize
) -> Vec<T> {
    // Collect all the results into a big sorted vec
    let mut giant_vec: Vec<T> = big_computation.collect();
    giant_vec.sort();
    // Return the last and therefore biggest n items with some iterator magic
    giant_vec.drain(..).rev().take(n).rev().collect()
}

```

But this is massively inefficient in two ways:
- Sorting very large lists is very slow, and we are sorting potentially billions of items that
  we will never need.
- For any decently large search space, storing these items will likely crash the computer by
  making it run out of memory.

## The solution used by this crate
This is where using a `Shortlist` is useful.

A `Shortlist` is a datastructure that will dynamically keep a 'shortlist' of the best items
given to it so far, with `O(1)` amortized time for pushing new items.  It will also only perform
one heap allocation when the `Shortlist` is created and every subsequent operation will be
allocation free.  Therefore, to the user of this library the code becomes:
```rust
use shortlist::Shortlist;

fn get_best<T: Ord>(
    big_computation: impl Iterator<Item = T>,
    n: usize
) -> Vec<T> {
    // Create a new Shortlist that will take at most `n` items
    let mut shortlist = Shortlist::new(n);
    // Feed it all the results from `big_computation`
    for v in big_computation {
        shortlist.push(v);
    }
    // Return the shortlisted values as a sorted vec
    shortlist.into_sorted_vec()
}

```

Or as a one-liner:
```rust
use shortlist::Shortlist;

fn get_best<T: Ord>(big_computation: impl Iterator<Item = T>, n: usize) -> Vec<T> {
    Shortlist::from_iter(n, big_computation).into_sorted_vec()
}

```

In both cases, the code will make exactly one heap allocation (to reserve space for the
`Shortlist`).

License: MIT
