#![cfg(feature = "rayon")]

use crate::{ArrayVec, Array};
use rayon::iter::{
    plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer},
    FromParallelIterator, IndexedParallelIterator, IntoParallelIterator, ParallelExtend,
    ParallelIterator,
};

// Adapted from `rayon/srs/vec.rs`

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
