
use std::cmp;
use std::ops::{Bound, RangeBounds};
use std::ptr;
use std::slice;

// extra traits
use std::fmt;

#[cfg(feature="std")]
use std::io;

use std::mem::MaybeUninit;


use crate::utils::MakeMaybeUninit;
use crate::{LenUint, CapacityError};
use crate::arrayvec_impl::ArrayVecImpl;

/// A vector with a fixed capacity that implements Copy.
///
/// The `ArrayVecCopy` is a vector backed by a fixed size array. It keeps track of
/// the number of initialized elements. The `ArrayVecCopy<T, CAP>` is parameterized
/// by `T` for the element type and `CAP` for the maximum capacity.
///
/// `CAP` is of type `usize` but is range limited to `u32::MAX`; attempting to create larger
/// ArrayVecCopys with larger capacity will panic.
///
/// The vector is a contiguous value (storing the elements inline) that you can store directly on
/// the stack if needed.
///
/// It offers a simple API but also dereferences to a slice, so that the full slice API is
/// available. The ArrayVecCopy can be converted into a by value iterator.
#[derive(Copy)]
pub struct ArrayVecCopy<T: Copy, const CAP: usize> {
    // the `len` first elements of the array are initialized
    pub(crate) xs: [MaybeUninit<T>; CAP],
    pub(crate) len: LenUint,
}

impl<T: Copy, const CAP: usize> ArrayVecCopy<T, CAP> {
    pub const fn new_const() -> Self {
        assert_capacity_limit_const!(CAP);
        ArrayVecCopy { xs: MakeMaybeUninit::ARRAY, len: 0 }
    }

    pub const fn const_push(self, element: T) -> Self {
        if let Ok(s) = self.const_try_push(element) {
            s
        } else {
            panic!("Exceeded max capacity")
        }
    }

    pub const fn const_try_push(mut self, element: T) -> Result<Self, CapacityError<T>> {
        if self.len < Self::CAPACITY as u32 {
            unsafe {
                self = self.const_push_unchecked(element);
            }
            Ok(self)
        } else {
            Err(CapacityError::new(element))
        }
    }

    pub const unsafe fn const_push_unchecked(mut self, element: T) -> Self {
        let len = self.len as usize;
        debug_assert!(len < Self::CAPACITY);
        self.xs[len] = MaybeUninit::new(element);
        self.len += 1;
        self
    }

    pub(crate) fn drain_range(&mut self, start: usize, end: usize) -> Drain<T, CAP> {
        let len = self.len();

        // bounds check happens here (before length is changed!)
        let range_slice: *const _ = &self[start..end];

        // Calling `set_len` creates a fresh and thus unique mutable references, making all
        // older aliases we created invalid. So we cannot call that function.
        self.len = start as LenUint;

        unsafe {
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: (*range_slice).iter(),
                vec: self as *mut _,
            }
        }
    }

    /// Create a draining iterator that removes the specified range in the vector
    /// and yields the removed items from start to end. The element range is
    /// removed even if the iterator is not consumed until the end.
    ///
    /// Note: It is unspecified how many elements are removed from the vector,
    /// if the `Drain` value is leaked.
    ///
    /// **Panics** if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// ```
    /// use arrayvec::$array;
    ///
    /// let mut v1 = $array::from([1, 2, 3]);
    /// let v2: $array<_, 3> = v1.drain(0..2).collect();
    /// assert_eq!(&v1[..], &[3]);
    /// assert_eq!(&v2[..], &[1, 2]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<T, CAP>
        where R: RangeBounds<usize>
    {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitialized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.saturating_add(1),
        };
        let end = match range.end_bound() {
            Bound::Excluded(&j) => j,
            Bound::Included(&j) => j.saturating_add(1),
            Bound::Unbounded => len,
        };
        self.drain_range(start, end)
    }

    /// Return the number of elements in the `ArrayVecCopy`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize { self.len as usize }

    /// Returns whether the `ArrayVecCopy` is empty.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1]);
    /// array.pop();
    /// assert_eq!(array.is_empty(), true);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Return the capacity of the `ArrayVecCopy`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let array = ArrayVecCopy::from([1, 2, 3]);
    /// assert_eq!(array.capacity(), 3);
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize { CAP }

    /// Return true if the `ArrayVecCopy` is completely filled to its capacity, false otherwise.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 1>::new();
    /// assert!(!array.is_full());
    /// array.push(1);
    /// assert!(array.is_full());
    /// ```
    pub fn is_full(&self) -> bool { self.len() == self.capacity() }

    /// Returns the capacity left in the `ArrayVecCopy`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.remaining_capacity(), 1);
    /// ```
    pub fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }
}

impl<T: Copy, const CAP: usize> ArrayVecImpl for ArrayVecCopy<T, CAP> {
    type Item = T;
    const CAPACITY: usize = CAP;

    fn len(&self) -> usize { self.len() }
    fn len_mut(&mut self) -> &mut LenUint {
        &mut self.len
    }

    unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= CAP);
        self.len = length as LenUint;
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.xs.as_ptr() as _
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.xs.as_mut_ptr() as _
    }
}

/// Iterate the `ArrayVecCopy` with each element by value.
///
/// The vector is consumed by this operation.
///
/// ```
/// use arrayvec::copy::ArrayVecCopy;
///
/// for elt in ArrayVecCopy::from([1, 2, 3]) {
///     // ...
/// }
/// ```
impl<T: Copy, const CAP: usize> IntoIterator for ArrayVecCopy<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;
    fn into_iter(self) -> IntoIter<T, CAP> {
        IntoIter { index: 0, v: self, }
    }
}


/// By-value iterator for `ArrayVecCopy`.
pub struct IntoIter<T: Copy, const CAP: usize> {
    index: usize,
    v: ArrayVecCopy<T, CAP>,
}

impl<T: Copy, const CAP: usize> Iterator for IntoIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let index = self.index;
                self.index = index + 1;
                Some(ptr::read(self.v.get_unchecked_ptr(index)))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len() - self.index;
        (len, Some(len))
    }
}

impl<T: Copy, const CAP: usize> DoubleEndedIterator for IntoIter<T, CAP> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let new_len = self.v.len() - 1;
                self.v.set_len(new_len);
                Some(ptr::read(self.v.get_unchecked_ptr(new_len)))
            }
        }
    }
}

impl<T: Copy, const CAP: usize> ExactSizeIterator for IntoIter<T, CAP> { }

impl<T: Copy, const CAP: usize> Drop for IntoIter<T, CAP> {
    fn drop(&mut self) {
        // panic safety: Set length to 0 before dropping elements.
        let index = self.index;
        let len = self.v.len();
        unsafe {
            self.v.set_len(0);
            let elements = slice::from_raw_parts_mut(
                self.v.get_unchecked_ptr(index),
                len - index);
            ptr::drop_in_place(elements);
        }
    }
}

impl<'a, T: 'a + Copy, const CAP: usize> Drop for Drain<'a, T, CAP> {
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
                let src = source_vec.as_ptr().add(tail);
                let dst = source_vec.as_mut_ptr().add(start);
                ptr::copy(src, dst, self.tail_len);
                source_vec.set_len(start + self.tail_len);
            }
        }
    }
}

impl<T: Copy, const CAP: usize> Clone for IntoIter<T, CAP> {
    fn clone(&self) -> IntoIter<T, CAP> {
        let mut v = ArrayVecCopy::new();
        v.extend_from_slice(&self.v[self.index..]);
        v.into_iter()
    }
}

impl<T: Copy, const CAP: usize> fmt::Debug for IntoIter<T, CAP>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(&self.v[self.index..])
            .finish()
    }
}

/// A draining iterator for `ArrayVecCopy`.
pub struct Drain<'a, T: 'a + Copy, const CAP: usize> {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'a, T>,
    vec: *mut ArrayVecCopy<T, CAP>,
}

unsafe impl<'a, T: Sync + Copy, const CAP: usize> Sync for Drain<'a, T, CAP> {}
unsafe impl<'a, T: Send + Copy, const CAP: usize> Send for Drain<'a, T, CAP> {}

impl<'a, T: 'a + Copy, const CAP: usize> Iterator for Drain<'a, T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|elt|
            unsafe {
                ptr::read(elt as *const _)
            }
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: 'a + Copy, const CAP: usize> DoubleEndedIterator for Drain<'a, T, CAP>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|elt|
            unsafe {
                ptr::read(elt as *const _)
            }
        )
    }
}

impl<'a, T: 'a + Copy, const CAP: usize> ExactSizeIterator for Drain<'a, T, CAP> {}

impl<T: Copy, const CAP: usize> fmt::Debug for ArrayVecCopy<T, CAP> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

impl<T: Copy, const CAP: usize> Default for ArrayVecCopy<T, CAP> {
    /// Return an empty array
    fn default() -> ArrayVecCopy<T, CAP> {
        ArrayVecCopy::new()
    }
}

#[cfg(feature="std")]
/// `Write` appends written data to the end of the vector.
///
/// Requires `features="std"`.
impl<const CAP: usize> io::Write for ArrayVecCopy<u8, CAP> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let len = cmp::min(self.remaining_capacity(), data.len());
        let _result = self.try_extend_from_slice(&data[..len]);
        debug_assert!(_result.is_ok());
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
