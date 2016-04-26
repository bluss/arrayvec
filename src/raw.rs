use CapacityError;
use RangeArgument;
use array::Array;
use std::borrow::{Borrow, BorrowMut};
use std::cmp;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;

#[cfg(feature="std")]
use std::io;

use array::Index;

pub struct RawArrayVec<A: Array> {
    xs: A,
    len: A::Index,
}

impl<A: Array> RawArrayVec<A> {
    #[inline]
    pub fn new() -> RawArrayVec<A> {
        unsafe {
            RawArrayVec {
                xs: mem::uninitialized(),
                len: Index::from(0),
            }
        }
    }

    #[inline] pub fn len(&self) -> usize { self.len.to_usize() }
    #[inline] pub fn capacity(&self) -> usize { A::capacity() }
    #[inline] pub fn is_full(&self) -> bool { self.len() == self.capacity() }

    pub fn push(&mut self, element: A::Item) -> Result<(), CapacityError<A::Item>> {
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

    pub fn insert(&mut self, index: usize, element: A::Item)
        -> Result<(), CapacityError<A::Item>>
    {
        assert!(index <= self.len());
        if index == self.capacity() {
            return Err(CapacityError::new(element));
        }
        let ret = if self.len() == self.capacity() {
            Err(CapacityError::new(self.pop().unwrap()))
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

    pub fn pop(&mut self) -> Option<A::Item> {
        if self.len() == 0 {
            return None
        }
        unsafe {
            let new_len = self.len() - 1;
            self.set_len(new_len);
            Some(ptr::read(self.get_unchecked_mut(new_len)))
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<A::Item> {
        let len = self.len();
        if index >= len {
            return None
        }
        self.swap(index, len - 1);
        self.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<A::Item> {
        if index >= self.len() {
            None
        } else {
            self.drain(index..index + 1).next()
        }
    }

    pub fn clear(&mut self) {
        while let Some(_) = self.pop() { }
    }

    pub fn retain<F>(&mut self, mut f: F)
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
            self.drain(len - del..);
        }
    }

    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= self.capacity());
        self.len = Index::from(length);
    }

    pub fn drain<R: RangeArgument>(&mut self, range: R) -> Drain<A> {
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
                vec: self as *mut _,
            }
        }
    }

    pub fn into_inner(self) -> Result<A, Self> {
        if self.len() < self.capacity() {
            Err(self)
        } else {
            unsafe {
                let array = ptr::read(&self.xs);
                mem::forget(self);
                Ok(array)
            }
        }
    }

    #[inline] pub fn dispose(self) { }
    #[inline] pub fn as_slice(&self) -> &[A::Item] { self }
    #[inline] pub fn as_mut_slice(&mut self) -> &mut [A::Item] { self }
}

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
    vec: *mut RawArrayVec<A>,
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
                let source_vec = &mut *self.vec;
                // memmove back untouched tail, update to new length
                let start = source_vec.len();
                let tail = self.tail_start;
                let src = source_vec.as_ptr().offset(tail as isize);
                let dst = source_vec.as_mut_ptr().offset(start as isize);
                ptr::copy(src, dst, self.tail_len);
                source_vec.set_len(start + self.tail_len);
            }
        }
    }
}

impl<A: Array> Extend<A::Item> for RawArrayVec<A> {
    fn extend<T: IntoIterator<Item=A::Item>>(&mut self, iter: T) {
        let take = self.capacity() - self.len();
        for elt in iter.into_iter().take(take) {
            let _ = self.push(elt);
        }
    }
}

impl<A: Array> iter::FromIterator<A::Item> for RawArrayVec<A> {
    fn from_iter<T: IntoIterator<Item=A::Item>>(iter: T) -> Self {
        let mut array = RawArrayVec::new();
        array.extend(iter);
        array
    }
}

impl<A: Array + Copy> Copy for RawArrayVec<A> where A::Item: Clone { }

impl<A: Array> Clone for RawArrayVec<A>
    where A::Item: Clone
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, rhs: &Self) {
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
                self.pop();
            }
        } else {
            for elt in &rhs[self.len()..] {
                let _ = self.push(elt.clone());
            }
        }
    }
}

impl<A: Array> Hash for RawArrayVec<A>
    where A::Item: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<A: Array> PartialEq for RawArrayVec<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<A: Array> PartialEq<[A::Item]> for RawArrayVec<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &[A::Item]) -> bool {
        **self == *other
    }
}

impl<A: Array> Eq for RawArrayVec<A> where A::Item: Eq { }

impl<A: Array> Borrow<[A::Item]> for RawArrayVec<A> {
    fn borrow(&self) -> &[A::Item] { self }
}

impl<A: Array> BorrowMut<[A::Item]> for RawArrayVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] { self }
}

impl<A: Array> AsRef<[A::Item]> for RawArrayVec<A> {
    fn as_ref(&self) -> &[A::Item] { self }
}

impl<A: Array> AsMut<[A::Item]> for RawArrayVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] { self }
}

impl<A: Array> fmt::Debug for RawArrayVec<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

impl<A: Array> Default for RawArrayVec<A> {
    fn default() -> RawArrayVec<A> {
        RawArrayVec::new()
    }
}

impl<A: Array> PartialOrd for RawArrayVec<A> where A::Item: PartialOrd {
    #[inline]
    fn partial_cmp(&self, other: &RawArrayVec<A>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }

    #[inline] fn lt(&self, other: &Self) -> bool { (**self).lt(other) }
    #[inline] fn le(&self, other: &Self) -> bool { (**self).le(other) }
    #[inline] fn ge(&self, other: &Self) -> bool { (**self).ge(other) }
    #[inline] fn gt(&self, other: &Self) -> bool { (**self).gt(other) }
}

impl<A: Array> Ord for RawArrayVec<A> where A::Item: Ord {
    fn cmp(&self, other: &RawArrayVec<A>) -> cmp::Ordering {
        (**self).cmp(other)
    }
}

#[cfg(feature="std")]
impl<A: Array<Item=u8>> io::Write for RawArrayVec<A> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
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
    #[inline]
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
