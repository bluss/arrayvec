use CapacityError;
use RangeArgument;
use array::Array;
use std::cmp;
use std::ops::{DerefMut};
use std::ptr;
use std::slice;

#[cfg(feature="std")]
use std::io;

use array::Index;

pub trait RawArrayVec<A: Array> : DerefMut<Target=[A::Item]> {
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn is_full_impl(&self) -> bool { self.len() == self.capacity() }
    unsafe fn set_len(&mut self, length: usize);
    fn len_ref(&mut self) -> &mut A::Index;


    fn push_impl(&mut self, element: A::Item) -> Result<(), CapacityError<A::Item>> {
        if self.len() < A::capacity() {
            let len = self.len();
            unsafe {
                ptr::write(self.get_unchecked_mut(len), element);
                self.set_len(len + 1);
            }
            Ok(())
        } else {
            Err(CapacityError::new(element))
        }
    }

    fn insert_impl(&mut self, index: usize, element: A::Item)
        -> Result<(), CapacityError<A::Item>>
    {
        assert!(index <= self.len());
        if index == self.capacity() {
            return Err(CapacityError::new(element));
        }
        let ret = if self.len() == self.capacity() {
            Err(CapacityError::new(self.pop_impl().unwrap()))
        } else {
            Ok(())
        };
        let len = self.len();

        // follows is just like Vec<T>
        unsafe { // infallible
            // The spot to put the new value
            {
                let p = self.get_unchecked_mut(index) as *mut _;
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
        ret
    }

    fn pop_impl(&mut self) -> Option<A::Item> {
        if self.len() == 0 {
            return None
        }
        unsafe {
            let new_len = self.len() - 1;
            self.set_len(new_len);
            Some(ptr::read(self.get_unchecked_mut(new_len)))
        }
    }

    fn swap_remove_impl(&mut self, index: usize) -> Option<A::Item> {
        let len = self.len();
        if index >= len {
            return None
        }
        self.swap(index, len - 1);
        self.pop_impl()
    }

    fn remove_impl(&mut self, index: usize) -> Option<A::Item> {
        if index >= self.len() {
            None
        } else {
            self.drain_impl(index..index + 1).next()
        }
    }

    fn clear_impl(&mut self) {
        while let Some(_) = self.pop_impl() { }
    }

    fn retain_impl<F>(&mut self, mut f: F)
        where F: FnMut(&mut A::Item) -> bool
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.drain_impl(len - del..);
        }
    }

    fn extend_impl<T: IntoIterator<Item=A::Item>>(&mut self, iter: T) {
        let take = self.capacity() - self.len();
        for elt in iter.into_iter().take(take) {
            let _ = self.push_impl(elt);
        }
    }

    #[cfg(feature="std")]
    fn write_impl(&mut self, data: &[u8]) -> io::Result<usize>
        where A: Array<Item=u8>
    {
        use std::io::Write;
        unsafe {
            let len = self.len();
            let mut tail = slice::from_raw_parts_mut(self.get_unchecked_mut(len),
                                                     A::capacity() - len);
            let result = tail.write(data);
            if let Ok(written) = result {
                self.set_len(len + written);
            }
            result
        }
    }

    fn drain_impl<R: RangeArgument>(&mut self, range: R) -> Drain<A> {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitalized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = self.len();
        let start = range.start().unwrap_or(0);
        let end = range.end().unwrap_or(len);
        // bounds check happens here
        let range_slice: *const _ = &self[start..end];

        unsafe {
            // set self.vec length's to start, to be safe in case Drain is leaked
            self.set_len(start);
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: (*range_slice).iter(),
                source_ptr: self.as_mut_ptr(),
                source_len: self.len_ref() as *mut _
            }
        }
    }

    fn clone_from_impl(&mut self, rhs: &Self)
        where A::Item: Clone
    {
        // recursive case for the common prefix
        let prefix = cmp::min(self.len(), rhs.len());
        {
            let a = &mut self[..prefix];
            let b = &rhs[..prefix];
            for i in 0..prefix {
                a[i].clone_from(&b[i]);
            }
        }
        if prefix < self.len() {
            // rhs was shorter
            for _ in 0..self.len() - prefix {
                self.pop_impl();
            }
        } else {
            for elt in &rhs[self.len()..] {
                let _ = self.push_impl(elt.clone());
            }
        }
    }
}

/*
impl<A: Array> Deref for RawArrayVec<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        unsafe {
            slice::from_raw_parts(self.xs.as_ptr(), self.len())
        }
    }
}

impl<A: Array> DerefMut for RawArrayVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts_mut(self.xs.as_mut_ptr(), len)
        }
    }
}

impl<A: Array> From<A> for RawArrayVec<A> {
    fn from(array: A) -> Self {
        RawArrayVec {
            xs: array,
            len: Index::from(A::capacity()),
        }
    }
}

impl<'a, A: Array> IntoIterator for &'a RawArrayVec<A> {
    type Item = &'a A::Item;
    type IntoIter = slice::Iter<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, A: Array> IntoIterator for &'a mut RawArrayVec<A> {
    type Item = &'a mut A::Item;
    type IntoIter = slice::IterMut<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

*/

/// A draining iterator for `ArrayVec`.
pub struct Drain<'a, A> 
    where A: Array,
          A::Item: 'a,
{
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'a, A::Item>,
    source_ptr: *mut A::Item,
    source_len: *mut A::Index,
}

unsafe impl<'a, A: Array + Sync> Sync for Drain<'a, A> {}
unsafe impl<'a, A: Array + Send> Send for Drain<'a, A> {}

impl<'a, A: Array> Iterator for Drain<'a, A>
    where A::Item: 'a,
{
    type Item = A::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|elt|
            unsafe {
                ptr::read(elt as *const _)
            }
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, A: Array> DoubleEndedIterator for Drain<'a, A>
    where A::Item: 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|elt|
            unsafe {
                ptr::read(elt as *const _)
            }
        )
    }
}

impl<'a, A: Array> ExactSizeIterator for Drain<'a, A> where A::Item: 'a {}

impl<'a, A: Array> Drop for Drain<'a, A>
    where A::Item: 'a
{
    fn drop(&mut self) {
        // len is currently 0 so panicking while dropping will not cause a double drop.

        // exhaust self first
        while let Some(_) = self.next() { }

        if self.tail_len > 0 {
            unsafe {
                let ptr = self.source_ptr;
                let source_len = &mut *self.source_len;
                // memmove back untouched tail, update to new length
                let start = source_len.to_usize();
                let tail = self.tail_start;
                let src = ptr.offset(tail as isize);
                let dst = ptr.offset(start as isize);
                ptr::copy(src, dst, self.tail_len);
                *source_len = Index::from(start + self.tail_len);
            }
        }
    }
}

