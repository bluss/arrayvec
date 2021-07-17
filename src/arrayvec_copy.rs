
use std::cmp;
use std::iter;
use std::mem;
use std::ops::{Bound, Deref, DerefMut, RangeBounds};
use std::ptr;
use std::slice;

// extra traits
use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};
use std::fmt;

#[cfg(feature="std")]
use std::io;

use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;

#[cfg(feature="serde")]
use serde::{Serialize, Deserialize, Serializer, Deserializer};

use crate::LenUint;
use crate::errors::CapacityError;
use crate::arrayvec_impl::ArrayVecImpl;

/// A vector with a fixed capacity which implements `Copy` and its elemenents are constrained to also be `Copy`.
///
/// The `ArrayVecCopy` is a vector backed by a fixed size array. It keeps track of
/// the number of initialized elements. The `ArrayVecCopy<T, CAP>` is parameterized
/// by `T` for the element type and `CAP` for the maximum capacity.
///
/// `CAP` is of type `usize` but is range limited to `u32::MAX`; attempting to create larger
/// arrayvecs with larger capacity will panic.
///
/// The vector is a contiguous value (storing the elements inline) that you can store directly on
/// the stack if needed.
///
/// It offers a simple API but also dereferences to a slice, so that the full slice API is
/// available. The ArrayVecCopy can be converted into a by value iterator.
pub struct ArrayVecCopy<T: Copy, const CAP: usize> {
    // the `len` first elements of the array are initialized
    xs: [MaybeUninit<T>; CAP],
    len: LenUint,
}

impl<T: Copy, const CAP: usize> Copy for ArrayVecCopy<T, CAP> {}

macro_rules! panic_oob {
    ($method_name:expr, $index:expr, $len:expr) => {
        panic!(concat!("ArrayVecCopy::", $method_name, ": index {} is out of bounds in vector of length {}"),
               $index, $len)
    }
}

impl<T: Copy, const CAP: usize> ArrayVecCopy<T, CAP> {
    /// Capacity
    const CAPACITY: usize = CAP;

    /// Create a new empty `ArrayVecCopy`.
    ///
    /// The maximum capacity is given by the generic parameter `CAP`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 16>::new();
    /// array.push(1);
    /// array.push(2);
    /// assert_eq!(&array[..], &[1, 2]);
    /// assert_eq!(array.capacity(), 16);
    /// ```
    pub fn new() -> ArrayVecCopy<T, CAP> {
        assert_capacity_limit!(CAP);
        unsafe {
            ArrayVecCopy { xs: MaybeUninit::uninit().assume_init(), len: 0 }
        }
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

    /// Push `element` to the end of the vector.
    ///
    /// ***Panics*** if the vector is already full.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// array.push(1);
    /// array.push(2);
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    /// ```
    pub fn push(&mut self, element: T) {
        ArrayVecImpl::push(self, element)
    }

    /// Push `element` to the end of the vector.
    ///
    /// Return `Ok` if the push succeeds, or return an error if the vector
    /// is already full.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// let push1 = array.try_push(1);
    /// let push2 = array.try_push(2);
    ///
    /// assert!(push1.is_ok());
    /// assert!(push2.is_ok());
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    ///
    /// let overflow = array.try_push(3);
    ///
    /// assert!(overflow.is_err());
    /// ```
    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        ArrayVecImpl::try_push(self, element)
    }

    /// Push `element` to the end of the vector without checking the capacity.
    ///
    /// It is up to the caller to ensure the capacity of the vector is
    /// sufficiently large.
    ///
    /// This method uses *debug assertions* to check that the arrayvec is not full.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// if array.len() + 2 <= array.capacity() {
    ///     unsafe {
    ///         array.push_unchecked(1);
    ///         array.push_unchecked(2);
    ///     }
    /// }
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    /// ```
    pub unsafe fn push_unchecked(&mut self, element: T) {
        ArrayVecImpl::push_unchecked(self, element)
    }

    /// Shortens the vector, keeping the first `len` elements.
    ///
    /// If `len` is greater than the vector’s current length this has no
    /// effect.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3, 4, 5]);
    /// array.truncate(3);
    /// assert_eq!(&array[..], &[1, 2, 3]);
    /// array.truncate(4);
    /// assert_eq!(&array[..], &[1, 2, 3]);
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        ArrayVecImpl::truncate(self, new_len)
    }

    /// Remove all elements in the vector.
    pub fn clear(&mut self) {
        ArrayVecImpl::clear(self)
    }


    /// Get pointer to where element at `index` would be
    unsafe fn get_unchecked_ptr(&mut self, index: usize) -> *mut T {
        self.as_mut_ptr().add(index)
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`.
    ///
    /// It is an error if the index is greater than the length or if the
    /// arrayvec is full.
    ///
    /// ***Panics*** if the array is full or the `index` is out of bounds. See
    /// `try_insert` for fallible version.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// array.insert(0, "x");
    /// array.insert(0, "y");
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        self.try_insert(index, element).unwrap()
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`; the `index` must be less than
    /// or equal to the length.
    ///
    /// Returns an error if vector is already at full capacity.
    ///
    /// ***Panics*** `index` is out of bounds.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// assert!(array.try_insert(0, "x").is_ok());
    /// assert!(array.try_insert(0, "y").is_ok());
    /// assert!(array.try_insert(0, "z").is_err());
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), CapacityError<T>> {
        if index > self.len() {
            panic_oob!("try_insert", index, self.len())
        }
        if self.len() == self.capacity() {
            return Err(CapacityError::new(element));
        }
        let len = self.len();

        // follows is just like Vec<T>
        unsafe { // infallible
            // The spot to put the new value
            {
                let p: *mut _ = self.get_unchecked_ptr(index);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
        Ok(())
    }

    /// Remove the last element in the vector and return it.
    ///
    /// Return `Some(` *element* `)` if the vector is non-empty, else `None`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::<_, 2>::new();
    ///
    /// array.push(1);
    ///
    /// assert_eq!(array.pop(), Some(1));
    /// assert_eq!(array.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        ArrayVecImpl::pop(self)
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return the *element* if the index is in bounds, else panic.
    ///
    /// ***Panics*** if the `index` is out of bounds.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_remove(0), 1);
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_remove(1), 2);
    /// assert_eq!(&array[..], &[3]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.swap_pop(index)
            .unwrap_or_else(|| {
                panic_oob!("swap_remove", index, self.len())
            })
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This is a checked version of `.swap_remove`.  
    /// This operation is O(1).
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_pop(0), Some(1));
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_pop(10), None);
    /// ```
    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        self.swap(index, len - 1);
        self.pop()
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// The `index` must be strictly less than the length of the vector.
    ///
    /// ***Panics*** if the `index` is out of bounds.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    ///
    /// let removed_elt = array.remove(0);
    /// assert_eq!(removed_elt, 1);
    /// assert_eq!(&array[..], &[2, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        self.pop_at(index)
            .unwrap_or_else(|| {
                panic_oob!("remove", index, self.len())
            })
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// This is a checked version of `.remove(index)`. Returns `None` if there
    /// is no element at `index`. Otherwise, return the element inside `Some`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3]);
    ///
    /// assert!(array.pop_at(0).is_some());
    /// assert_eq!(&array[..], &[2, 3]);
    ///
    /// assert!(array.pop_at(2).is_none());
    /// assert!(array.pop_at(10).is_none());
    /// ```
    pub fn pop_at(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            self.drain(index..index + 1).next()
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns false.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut array = ArrayVecCopy::from([1, 2, 3, 4]);
    /// array.retain(|x| *x & 1 != 0 );
    /// assert_eq!(&array[..], &[1, 3]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&mut T) -> bool
    {
        // Check the implementation of
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
        // for safety arguments (especially regarding panics in f and when
        // dropping elements). Implementation closely mirrored here.

        let original_len = self.len();
        unsafe { self.set_len(0) };

        struct BackshiftOnDrop<'a, T: Copy, const CAP: usize> {
            v: &'a mut ArrayVecCopy<T, CAP>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T: Copy, const CAP: usize> Drop for BackshiftOnDrop<'_, T, CAP> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v.as_mut_ptr().add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len
                        );
                    }
                }
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop { v: self, processed_len: 0, deleted_cnt: 0, original_len };

        while g.processed_len < original_len {
            let cur = unsafe { g.v.as_mut_ptr().add(g.processed_len) };
            if !f(unsafe { &mut *cur }) {
                g.processed_len += 1;
                g.deleted_cnt += 1;
                unsafe { ptr::drop_in_place(cur) };
                continue;
            }
            if g.deleted_cnt > 0 {
                unsafe {
                    let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                    ptr::copy_nonoverlapping(cur, hole_slot, 1);
                }
            }
            g.processed_len += 1;
        }

        drop(g);
    }

    /// Set the vector’s length without dropping or moving out elements
    ///
    /// This method is `unsafe` because it changes the notion of the
    /// number of “valid” elements in the vector. Use with care.
    ///
    /// This method uses *debug assertions* to check that `length` is
    /// not greater than the capacity.
    pub unsafe fn set_len(&mut self, length: usize) {
        // type invariant that capacity always fits in LenUint
        debug_assert!(length <= self.capacity());
        self.len = length as LenUint;
    }

    /// Copy all elements from the slice and append to the `ArrayVecCopy`.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut vec: ArrayVecCopy<usize, 10> = ArrayVecCopy::new();
    /// vec.push(1);
    /// vec.try_extend_from_slice(&[2, 3]).unwrap();
    /// assert_eq!(&vec[..], &[1, 2, 3]);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if the capacity left (see
    /// [`remaining_capacity`]) is smaller then the length of the provided
    /// slice.
    ///
    /// [`remaining_capacity`]: #method.remaining_capacity
    pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError> {
        if self.remaining_capacity() < other.len() {
            return Err(CapacityError::new(()));
        }

        let self_len = self.len();
        let other_len = other.len();

        unsafe {
            let dst = self.get_unchecked_ptr(self_len);
            ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
            self.set_len(self_len + other_len);
        }
        Ok(())
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
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut v1 = ArrayVecCopy::from([1, 2, 3]);
    /// let v2: ArrayVecCopy<_, 3> = v1.drain(0..2).collect();
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

    fn drain_range(&mut self, start: usize, end: usize) -> Drain<T, CAP>
    {
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

    /// Return the inner fixed size array, if it is full to its capacity.
    ///
    /// Return an `Ok` value with the array if length equals capacity,
    /// return an `Err` with self otherwise.
    pub fn into_inner(self) -> Result<[T; CAP], Self> {
        if self.len() < self.capacity() {
            Err(self)
        } else {
            unsafe { Ok(self.into_inner_unchecked()) }
        }
    }

    /// Return the inner fixed size array.
    ///
    /// Safety:
    /// This operation is safe if and only if length equals capacity.
    pub unsafe fn into_inner_unchecked(self) -> [T; CAP] {
        debug_assert_eq!(self.len(), self.capacity());
        let self_ = ManuallyDrop::new(self);
        let array = ptr::read(self_.as_ptr() as *const [T; CAP]);
        array
    }

    /// Returns the ArrayVecCopy, replacing the original with a new empty ArrayVecCopy.
    ///
    /// ```
    /// use arrayvec::ArrayVecCopy;
    ///
    /// let mut v = ArrayVecCopy::from([0, 1, 2, 3]);
    /// assert_eq!([0, 1, 2, 3], v.take().into_inner().unwrap());
    /// assert!(v.is_empty());
    /// ```
    pub fn take(&mut self) -> Self  {
        mem::replace(self, Self::new())
    }

    /// Return a slice containing all elements of the vector.
    pub fn as_slice(&self) -> &[T] {
        ArrayVecImpl::as_slice(self)
    }

    /// Return a mutable slice containing all elements of the vector.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        ArrayVecImpl::as_mut_slice(self)
    }

    /// Return a raw pointer to the vector's buffer.
    pub fn as_ptr(&self) -> *const T {
        ArrayVecImpl::as_ptr(self)
    }

    /// Return a raw mutable pointer to the vector's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        ArrayVecImpl::as_mut_ptr(self)
    }
}

impl<T: Copy, const CAP: usize> ArrayVecImpl for ArrayVecCopy<T, CAP> {
    type Item = T;
    const CAPACITY: usize = CAP;

    fn len(&self) -> usize { self.len() }

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

impl<T: Copy, const CAP: usize> Deref for ArrayVecCopy<T, CAP> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy, const CAP: usize> DerefMut for ArrayVecCopy<T, CAP> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}


/// Create an `ArrayVecCopy` from an array.
///
/// ```
/// use arrayvec::ArrayVecCopy;
///
/// let mut array = ArrayVecCopy::from([1, 2, 3]);
/// assert_eq!(array.len(), 3);
/// assert_eq!(array.capacity(), 3);
/// ```
impl<T: Copy, const CAP: usize> From<[T; CAP]> for ArrayVecCopy<T, CAP> {
    fn from(array: [T; CAP]) -> Self {
        let array = ManuallyDrop::new(array);
        let mut vec = <ArrayVecCopy<T, CAP>>::new();
        unsafe {
            (&*array as *const [T; CAP] as *const [MaybeUninit<T>; CAP])
                .copy_to_nonoverlapping(&mut vec.xs as *mut [MaybeUninit<T>; CAP], 1);
            vec.set_len(CAP);
        }
        vec
    }
}


/// Try to create an `ArrayVecCopy` from a slice. This will return an error if the slice was too big to
/// fit.
///
/// ```
/// use arrayvec::ArrayVecCopy;
/// use std::convert::TryInto as _;
///
/// let array: ArrayVecCopy<_, 4> = (&[1, 2, 3] as &[_]).try_into().unwrap();
/// assert_eq!(array.len(), 3);
/// assert_eq!(array.capacity(), 4);
/// ```
impl<T: Copy, const CAP: usize> std::convert::TryFrom<&[T]> for ArrayVecCopy<T, CAP> {
    type Error = CapacityError;

    fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
        if Self::CAPACITY < slice.len() {
            Err(CapacityError::new(()))
        } else {
            let mut array = Self::new();
            array.extend_from_slice(slice);
            Ok(array)
        }
    }
}


/// Iterate the `ArrayVecCopy` with references to each element.
///
/// ```
/// use arrayvec::ArrayVecCopy;
///
/// let array = ArrayVecCopy::from([1, 2, 3]);
///
/// for elt in &array {
///     // ...
/// }
/// ```
impl<'a, T: Copy + 'a, const CAP: usize> IntoIterator for &'a ArrayVecCopy<T, CAP> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// Iterate the `ArrayVecCopy` with mutable references to each element.
///
/// ```
/// use arrayvec::ArrayVecCopy;
///
/// let mut array = ArrayVecCopy::from([1, 2, 3]);
///
/// for elt in &mut array {
///     // ...
/// }
/// ```
impl<'a, T: Copy + 'a, const CAP: usize> IntoIterator for &'a mut ArrayVecCopy<T, CAP> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

/// Iterate the `ArrayVecCopy` with each element by value.
///
/// The vector is consumed by this operation.
///
/// ```
/// use arrayvec::ArrayVecCopy;
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
pub struct Drain<'a, T: Copy + 'a, const CAP: usize> {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'a, T>,
    vec: *mut ArrayVecCopy<T, CAP>,
}

unsafe impl<'a, T: Copy + Sync, const CAP: usize> Sync for Drain<'a, T, CAP> {}
unsafe impl<'a, T: Copy + Send, const CAP: usize> Send for Drain<'a, T, CAP> {}

impl<'a, T: Copy + 'a, const CAP: usize> Iterator for Drain<'a, T, CAP> {
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

impl<'a, T: Copy + 'a, const CAP: usize> DoubleEndedIterator for Drain<'a, T, CAP>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|elt|
            unsafe {
                ptr::read(elt as *const _)
            }
        )
    }
}

impl<'a, T: Copy + 'a, const CAP: usize> ExactSizeIterator for Drain<'a, T, CAP> {}

impl<'a, T: Copy + 'a, const CAP: usize> Drop for Drain<'a, T, CAP> {
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

struct ScopeExitGuard<T, Data, F>
    where F: FnMut(&Data, &mut T)
{
    value: T,
    data: Data,
    f: F,
}

impl<T, Data, F> Drop for ScopeExitGuard<T, Data, F>
    where F: FnMut(&Data, &mut T)
{
    fn drop(&mut self) {
        (self.f)(&self.data, &mut self.value)
    }
}



/// Extend the `ArrayVecCopy` with an iterator.
/// 
/// ***Panics*** if extending the vector exceeds its capacity.
impl<T: Copy, const CAP: usize> Extend<T> for ArrayVecCopy<T, CAP> {
    /// Extend the `ArrayVecCopy` with an iterator.
    /// 
    /// ***Panics*** if extending the vector exceeds its capacity.
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        unsafe {
            self.extend_from_iter::<_, true>(iter)
        }
    }
}

#[inline(never)]
#[cold]
fn extend_panic() {
    panic!("ArrayVecCopy: capacity exceeded in extend/from_iter");
}

impl<T: Copy, const CAP: usize> ArrayVecCopy<T, CAP> {
    /// Extend the arrayvec from the iterable.
    ///
    /// ## Safety
    ///
    /// Unsafe because if CHECK is false, the length of the input is not checked.
    /// The caller must ensure the length of the input fits in the capacity.
    pub(crate) unsafe fn extend_from_iter<I, const CHECK: bool>(&mut self, iterable: I)
        where I: IntoIterator<Item = T>
    {
        let take = self.capacity() - self.len();
        let len = self.len();
        let mut ptr = raw_ptr_add(self.as_mut_ptr(), len);
        let end_ptr = raw_ptr_add(ptr, take);
        // Keep the length in a separate variable, write it back on scope
        // exit. To help the compiler with alias analysis and stuff.
        // We update the length to handle panic in the iteration of the
        // user's iterator, without dropping any elements on the floor.
        let mut guard = ScopeExitGuard {
            value: &mut self.len,
            data: len,
            f: move |&len, self_len| {
                **self_len = len as LenUint;
            }
        };
        let mut iter = iterable.into_iter();
        loop {
            if let Some(elt) = iter.next() {
                if ptr == end_ptr && CHECK { extend_panic(); }
                debug_assert_ne!(ptr, end_ptr);
                ptr.write(elt);
                ptr = raw_ptr_add(ptr, 1);
                guard.data += 1;
            } else {
                return; // success
            }
        }
    }

    /// Extend the ArrayVecCopy with clones of elements from the slice;
    /// the length of the slice must be <= the remaining capacity in the arrayvec.
    pub(crate) fn extend_from_slice(&mut self, slice: &[T]) {
        let take = self.capacity() - self.len();
        debug_assert!(slice.len() <= take);
        unsafe {
            let slice = if take < slice.len() { &slice[..take] } else { slice };
            self.extend_from_iter::<_, false>(slice.iter().cloned());
        }
    }
}

/// Rawptr add but uses arithmetic distance for ZST
unsafe fn raw_ptr_add<T>(ptr: *mut T, offset: usize) -> *mut T {
    if mem::size_of::<T>() == 0 {
        // Special case for ZST
        (ptr as usize).wrapping_add(offset) as _
    } else {
        ptr.add(offset)
    }
}

/// Create an `ArrayVecCopy` from an iterator.
/// 
/// ***Panics*** if the number of elements in the iterator exceeds the arrayvec's capacity.
impl<T: Copy, const CAP: usize> iter::FromIterator<T> for ArrayVecCopy<T, CAP> {
    /// Create an `ArrayVecCopy` from an iterator.
    /// 
    /// ***Panics*** if the number of elements in the iterator exceeds the arrayvec's capacity.
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut array = ArrayVecCopy::new();
        array.extend(iter);
        array
    }
}

impl<T: Copy, const CAP: usize> Clone for ArrayVecCopy<T, CAP> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, rhs: &Self) {
        // recursive case for the common prefix
        let prefix = cmp::min(self.len(), rhs.len());
        self[..prefix].clone_from_slice(&rhs[..prefix]);

        if prefix < self.len() {
            // rhs was shorter
            self.truncate(prefix);
        } else {
            let rhs_elems = &rhs[self.len()..];
            self.extend_from_slice(rhs_elems);
        }
    }
}

impl<T: Copy, const CAP: usize> Hash for ArrayVecCopy<T, CAP>
    where T: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T: Copy, const CAP: usize> PartialEq for ArrayVecCopy<T, CAP>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Copy, const CAP: usize> PartialEq<[T]> for ArrayVecCopy<T, CAP>
    where T: PartialEq
{
    fn eq(&self, other: &[T]) -> bool {
        **self == *other
    }
}

impl<T: Copy, const CAP: usize> Eq for ArrayVecCopy<T, CAP> where T: Eq { }

impl<T: Copy, const CAP: usize> Borrow<[T]> for ArrayVecCopy<T, CAP> {
    fn borrow(&self) -> &[T] { self }
}

impl<T: Copy, const CAP: usize> BorrowMut<[T]> for ArrayVecCopy<T, CAP> {
    fn borrow_mut(&mut self) -> &mut [T] { self }
}

impl<T: Copy, const CAP: usize> AsRef<[T]> for ArrayVecCopy<T, CAP> {
    fn as_ref(&self) -> &[T] { self }
}

impl<T: Copy, const CAP: usize> AsMut<[T]> for ArrayVecCopy<T, CAP> {
    fn as_mut(&mut self) -> &mut [T] { self }
}

impl<T: Copy, const CAP: usize> fmt::Debug for ArrayVecCopy<T, CAP> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

impl<T: Copy, const CAP: usize> Default for ArrayVecCopy<T, CAP> {
    /// Return an empty array
    fn default() -> ArrayVecCopy<T, CAP> {
        ArrayVecCopy::new()
    }
}

impl<T: Copy, const CAP: usize> PartialOrd for ArrayVecCopy<T, CAP> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }

    fn lt(&self, other: &Self) -> bool {
        (**self).lt(other)
    }

    fn le(&self, other: &Self) -> bool {
        (**self).le(other)
    }

    fn ge(&self, other: &Self) -> bool {
        (**self).ge(other)
    }

    fn gt(&self, other: &Self) -> bool {
        (**self).gt(other)
    }
}

impl<T: Copy, const CAP: usize> Ord for ArrayVecCopy<T, CAP> where T: Ord {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (**self).cmp(other)
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

#[cfg(feature="serde")]
/// Requires crate feature `"serde"`
impl<T: Copy + Serialize, const CAP: usize> Serialize for ArrayVecCopy<T, CAP> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.collect_seq(self)
    }
}

#[cfg(feature="serde")]
/// Requires crate feature `"serde"`
impl<'de, T: Copy + Deserialize<'de>, const CAP: usize> Deserialize<'de> for ArrayVecCopy<T, CAP> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        use serde::de::{Visitor, SeqAccess, Error};
        use std::marker::PhantomData;

        struct ArrayVecCopyVisitor<'de, T: Copy + Deserialize<'de>, const CAP: usize>(PhantomData<(&'de (), [T; CAP])>);

        impl<'de, T: Copy + Deserialize<'de>, const CAP: usize> Visitor<'de> for ArrayVecCopyVisitor<'de, T, CAP> {
            type Value = ArrayVecCopy<T, CAP>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array with no more than {} items", CAP)
            }

            fn visit_seq<SA>(self, mut seq: SA) -> Result<Self::Value, SA::Error>
                where SA: SeqAccess<'de>,
            {
                let mut values = ArrayVecCopy::<T, CAP>::new();

                while let Some(value) = seq.next_element()? {
                    if let Err(_) = values.try_push(value) {
                        return Err(SA::Error::invalid_length(CAP + 1, &self));
                    }
                }

                Ok(values)
            }
        }

        deserializer.deserialize_seq(ArrayVecCopyVisitor::<T, CAP>(PhantomData))
    }
}
