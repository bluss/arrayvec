#![cfg(feature = "rayon")]

use crate::{Array, ArrayVec};
use rayon::iter::{
    plumbing::*, FromParallelIterator, IndexedParallelIterator, IntoParallelIterator,
    ParallelExtend, ParallelIterator,
};
use std::marker::PhantomData;
use std::{ptr, slice};

// Adapted from `rayon/src/vec.rs`

/// Parallel iterator that moves out of an `ArrayVec`.
#[derive(Debug, Clone)]
pub struct IntoParIter<T, A: Array<Item = T>> {
    vec: ArrayVec<A>,
}

impl<A> IntoParallelIterator for ArrayVec<A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    type Item = A::Item;
    type Iter = IntoParIter<A::Item, A>;

    fn into_par_iter(self) -> Self::Iter {
        IntoParIter { vec: self }
    }
}

impl<A> ParallelIterator for IntoParIter<A::Item, A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    type Item = A::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<A> IndexedParallelIterator for IntoParIter<A::Item, A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn with_producer<CB>(mut self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        // The producer will move or drop each item from its slice, effectively taking ownership of
        // them.  When we're done, the vector only needs to free its buffer.
        unsafe {
            // Make the `ArrayVec` forget about the actual items.
            let len = self.vec.len();
            self.vec.set_len(0);

            // Get a correct borrow, then extend it to the original length.
            let mut slice = self.vec.as_mut_slice();
            slice = std::slice::from_raw_parts_mut(slice.as_mut_ptr(), len);

            callback.callback(ArrayVecProducer { slice })
        }
    }
}

struct ArrayVecProducer<'data, T: Send> {
    slice: &'data mut [T],
}

impl<'data, T: 'data + Send> Producer for ArrayVecProducer<'data, T> {
    type Item = T;
    type IntoIter = SliceDrain<'data, T>;

    fn into_iter(mut self) -> Self::IntoIter {
        // replace the slice so we don't drop it twice
        let slice = std::mem::replace(&mut self.slice, &mut []);
        SliceDrain {
            iter: slice.iter_mut(),
        }
    }

    fn split_at(mut self, index: usize) -> (Self, Self) {
        // replace the slice so we don't drop it twice
        let slice = std::mem::replace(&mut self.slice, &mut []);
        let (left, right) = slice.split_at_mut(index);
        (
            ArrayVecProducer { slice: left },
            ArrayVecProducer { slice: right },
        )
    }
}

impl<'data, T: 'data + Send> Drop for ArrayVecProducer<'data, T> {
    fn drop(&mut self) {
        SliceDrain {
            iter: self.slice.iter_mut(),
        };
    }
}

/// ////////////////////////////////////////////////////////////////////////

// like std::vec::Drain, without updating a source Vec
struct SliceDrain<'data, T> {
    iter: std::slice::IterMut<'data, T>,
}

impl<'data, T: 'data> Iterator for SliceDrain<'data, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let ptr = self.iter.next()?;
        Some(unsafe { std::ptr::read(ptr) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'data, T: 'data> DoubleEndedIterator for SliceDrain<'data, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ptr = self.iter.next_back()?;
        Some(unsafe { std::ptr::read(ptr) })
    }
}

impl<'data, T: 'data> ExactSizeIterator for SliceDrain<'data, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'data, T: 'data> Drop for SliceDrain<'data, T> {
    fn drop(&mut self) {
        for ptr in &mut self.iter {
            unsafe {
                std::ptr::drop_in_place(ptr);
            }
        }
    }
}

// Adapted from `rayon/src/iter/collect/mod.rs` and `rayon/src/iter/collect/consumer.rs`

impl<A> FromParallelIterator<A::Item> for ArrayVec<A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = A::Item>,
    {
        let mut arrayvec = Self::new();
        arrayvec.par_extend(par_iter);
        arrayvec
    }
}

impl<A> ParallelExtend<A::Item> for ArrayVec<A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = A::Item>,
    {
        let par_iter = par_iter.into_par_iter();

        if let Some(len) = par_iter.opt_len() {
            // When Rust gets specialization, we can get here for indexed iterators
            // without relying on `opt_len`.  Until then, `special_extend()` fakes
            // an unindexed mode on the promise that `opt_len()` is accurate.
            Collect::new(self, len).with_consumer(|consumer| par_iter.drive_unindexed(consumer));
        } else {
            self.extend(
                par_iter
                    .into_par_iter()
                    .fold(
                        || Self::new(),
                        |mut arrayvec, element| {
                            let _ = arrayvec.try_push(element);
                            arrayvec
                        },
                    )
                    .reduce(Self::new, |mut arrayvec1, arrayvec2| {
                        // TODO: use `ArrayVec::append/try_append` when it becomes available
                        arrayvec1.extend(arrayvec2);
                        arrayvec1
                    }),
            )
        }
    }
}

/// Manage the collection vector.
struct Collect<'c, A: Array> {
    vec: &'c mut ArrayVec<A>,
    len: usize,
}

impl<'c, A> Collect<'c, A>
where
    A: Array + Send,
    A::Item: Send,
    A::Index: Send,
{
    fn new(vec: &'c mut ArrayVec<A>, len: usize) -> Self {
        Collect { vec, len }
    }

    /// Create a consumer on the slice of memory we are collecting into.
    ///
    /// The consumer needs to be used inside the scope function, and the
    /// complete collect result passed back.
    ///
    /// This method will verify the collect result, and panic if the slice
    /// was not fully written into. Otherwise, in the successful case,
    /// the vector is complete with the collected result.
    fn with_consumer<F>(mut self, scope_fn: F)
    where
        F: FnOnce(CollectConsumer<'_, A::Item>) -> CollectResult<'_, A::Item>,
    {
        unsafe {
            let slice = Self::reserve_get_tail_slice(&mut self.vec, self.len);
            let expected_writes = slice.len();
            let result = scope_fn(CollectConsumer::new(slice));

            // The CollectResult represents a contiguous part of the
            // slice, that has been written to.
            // On unwind here, the CollectResult will be dropped.
            // If some producers on the way did not produce enough elements,
            // partial CollectResults may have been dropped without
            // being reduced to the final result, and we will see
            // that as the length coming up short.
            //
            // Here, we assert that `slice` is fully initialized. This is
            // checked by the following assert, which verifies if a
            // complete CollectResult was produced; if the length is
            // correct, it is necessarily covering the target slice.
            // Since we know that the consumer cannot have escaped from
            // `drive` (by parametricity, essentially), we know that any
            // stores that will happen, have happened. Unless some code is buggy,
            // that means we should have seen `len` total writes.
            let actual_writes = result.len();
            assert!(
                actual_writes == expected_writes,
                "expected {} total writes, but got {}",
                expected_writes,
                actual_writes
            );

            // Release the result's mutable borrow and "proxy ownership"
            // of the elements, before the vector takes it over.
            result.release_ownership();

            let new_len = self.vec.len() + expected_writes;
            self.vec.set_len(new_len);
        }
    }

    /// Reserve space for `len` more elements in the vector,
    /// and return a slice to the uninitialized tail of the vector
    ///
    /// Safety: The tail slice is uninitialized
    unsafe fn reserve_get_tail_slice(vec: &mut ArrayVec<A>, len: usize) -> &mut [A::Item] {
        // Cap the slice length
        let actual_len = std::cmp::min(A::CAPACITY - vec.len(), len);
        // Get a correct borrow, then extend it for the newly added length.
        let start = vec.len();
        let slice = &mut vec[start..];
        slice::from_raw_parts_mut(slice.as_mut_ptr(), actual_len)
    }
}

pub(super) struct CollectConsumer<'c, T: Send> {
    /// A slice covering the target memory, not yet initialized!
    target: &'c mut [T],
}

pub(super) struct CollectFolder<'c, T: Send> {
    /// The folder writes into `result` and must extend the result
    /// up to exactly this number of elements.
    final_len: usize,

    /// The current written-to part of our slice of the target
    result: CollectResult<'c, T>,
}

impl<'c, T: Send + 'c> CollectConsumer<'c, T> {
    /// The target memory is considered uninitialized, and will be
    /// overwritten without reading or dropping existing values.
    pub(super) fn new(target: &'c mut [T]) -> Self {
        CollectConsumer { target }
    }
}

/// CollectResult represents an initialized part of the target slice.
///
/// This is a proxy owner of the elements in the slice; when it drops,
/// the elements will be dropped, unless its ownership is released before then.
#[must_use]
pub(super) struct CollectResult<'c, T> {
    start: *mut T,
    len: usize,
    invariant_lifetime: PhantomData<&'c mut &'c mut [T]>,
}

unsafe impl<'c, T> Send for CollectResult<'c, T> where T: Send {}

impl<'c, T> CollectResult<'c, T> {
    /// The current length of the collect result
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// Release ownership of the slice of elements, and return the length
    pub(super) fn release_ownership(mut self) -> usize {
        let ret = self.len;
        self.len = 0;
        ret
    }
}

impl<'c, T> Drop for CollectResult<'c, T> {
    fn drop(&mut self) {
        // Drop the first `self.len` elements, which have been recorded
        // to be initialized by the folder.
        unsafe {
            ptr::drop_in_place(slice::from_raw_parts_mut(self.start, self.len));
        }
    }
}

impl<'c, T: Send + 'c> Consumer<T> for CollectConsumer<'c, T> {
    type Folder = CollectFolder<'c, T>;
    type Reducer = CollectReducer;
    type Result = CollectResult<'c, T>;

    fn split_at(self, index: usize) -> (Self, Self, CollectReducer) {
        let CollectConsumer { target } = self;

        // Produce new consumers. Normal slicing ensures that the
        // memory range given to each consumer is disjoint.

        let (left, right) = if index < target.len() {
            target.split_at_mut(index)
        } else {
            (target, &mut [][..])
        };
        (
            CollectConsumer::new(left),
            CollectConsumer::new(right),
            CollectReducer,
        )
    }

    fn into_folder(self) -> CollectFolder<'c, T> {
        // Create a folder that consumes values and writes them
        // into target. The initial result has length 0.
        CollectFolder {
            final_len: self.target.len(),
            result: CollectResult {
                start: self.target.as_mut_ptr(),
                len: 0,
                invariant_lifetime: PhantomData,
            },
        }
    }

    fn full(&self) -> bool {
        self.target.len() == 0
    }
}

impl<'c, T: Send + 'c> Folder<T> for CollectFolder<'c, T> {
    type Result = CollectResult<'c, T>;

    fn consume(mut self, item: T) -> CollectFolder<'c, T> {
        if self.result.len >= self.final_len {
            panic!("too many values pushed to consumer");
        }

        // Compute target pointer and write to it, and
        // extend the current result by one element
        unsafe {
            self.result.start.add(self.result.len).write(item);
            self.result.len += 1;
        }

        self
    }

    fn complete(self) -> Self::Result {
        // NB: We don't explicitly check that the local writes were complete,
        // but Collect will assert the total result length in the end.
        self.result
    }

    fn full(&self) -> bool {
        self.result.len == self.final_len
    }
}

/// Pretend to be unindexed for `special_collect_into_vec`,
/// but we should never actually get used that way...
impl<'c, T: Send + 'c> UnindexedConsumer<T> for CollectConsumer<'c, T> {
    fn split_off_left(&self) -> Self {
        unreachable!("CollectConsumer must be indexed!")
    }
    fn to_reducer(&self) -> Self::Reducer {
        CollectReducer
    }
}

/// CollectReducer combines adjacent chunks; the result must always
/// be contiguous so that it is one combined slice.
pub(super) struct CollectReducer;

impl<'c, T> Reducer<CollectResult<'c, T>> for CollectReducer {
    fn reduce(
        self,
        mut left: CollectResult<'c, T>,
        right: CollectResult<'c, T>,
    ) -> CollectResult<'c, T> {
        // Merge if the CollectResults are adjacent and in left to right order
        // else: drop the right piece now and total length will end up short in the end,
        // when the correctness of the collected result is asserted.
        if left.start.wrapping_add(left.len) == right.start {
            left.len += right.release_ownership();
        }
        left
    }
}
