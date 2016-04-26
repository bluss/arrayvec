use std::cmp;
use std::iter;
use std::ptr;
use std::ops;
use std::slice;

// extra traits
use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};
use std::fmt;

#[cfg(feature="std")]
use std::io;

use Array;
use CapacityError;
use RangeArgument;
use array::Index;
use raw::Drain;
use raw::RawArrayVec;

/// A vector with a fixed capacity that implements `Copy`.
pub struct ArrayVecCopy<A: Array + Copy> {
    inner: RawArrayVec<A>,
}

impl<A: Array + Copy> ArrayVecCopy<A> {
    /// Create a new empty `ArrayVecCopy`.
    ///
    /// Capacity is inferred from the type parameter.
    #[inline]
    pub fn new() -> ArrayVecCopy<A> {
        ArrayVecCopy {
            inner: RawArrayVec::new(),
        }
    }

    /// Return the number of elements in the `ArrayVecCopy`.
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Return the capacity of the `ArrayVecCopy`.
    #[inline]
    pub fn capacity(&self) -> usize { self.inner.capacity() }

    /// Push `element` to the end of the vector.
    ///
    /// Returns `Ok` if the push succeeds.
    ///
    /// **Errors** if the backing array is not large enough to fit the
    /// additional element.
    #[inline]
    pub fn push(&mut self, element: A::Item) -> Result<(), CapacityError<A::Item>> {
        self.inner.push(element)
    }

    /// Insert `element` in position `index`.
    ///
    /// Shift up all elements after `index`. If any is pushed out, `Err` is
    /// returned.
    ///
    /// Return `Ok` if no element is shifted out.
    ///
    /// **Panics** if the specified index is greater than the current length.
    #[inline]
    pub fn insert(&mut self, index: usize, element: A::Item)
        -> Result<(), CapacityError<A::Item>>
    {
        self.inner.insert(index, element)
    }

    /// Remove the last element in the vector.
    ///
    /// Return `Some(` *element* `)` if the vector is non-empty, else `None`.
    #[inline]
    pub fn pop(&mut self) -> Option<A::Item> {
        self.inner.pop()
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> Option<A::Item> {
        self.inner.swap_remove(index)
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<A::Item> {
        self.inner.remove(index)
    }

    /// Remove all elements in the vector.
    ///
    /// This is a constant-time operation.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            self.set_len(0);
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns false.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
        where F: FnMut(&mut A::Item) -> bool
    {
        self.inner.retain(f)
    }

    /// Set the vector's length without dropping or moving out elements
    ///
    /// May panic if `length` is greater than the capacity.
    ///
    /// This function is `unsafe` because it changes the notion of the
    /// number of “valid” elements in the vector. Use with care.
    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.inner.set_len(length)
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
    pub fn drain<R: RangeArgument>(&mut self, range: R) -> Drain<A> {
        self.inner.drain(range)
    }

    /// Return the inner fixed size array, if it is full to its capacity.
    ///
    /// Return an `Ok` value with the array if length equals capacity,
    /// return an `Err` with self otherwise.
    ///
    /// `Note:` This function may incur unproportionally large overhead
    /// to move the array out, its performance is not optimal.
    pub fn into_inner(self) -> Result<A, Self> {
        self.inner.into_inner().map_err(|e| ArrayVecCopy { inner: e })
    }

    /// Dispose of `self` without the overwriting that is needed in Drop.
    pub fn dispose(self) { }

    /// Return a slice containing all elements of the vector.
    pub fn as_slice(&self) -> &[A::Item] {
        self.inner.as_slice()
    }

    /// Return a mutable slice containing all elements of the vector.
    pub fn as_mut_slice(&mut self) -> &mut [A::Item] {
        self.inner.as_mut_slice()
    }
}

impl<A: Array + Copy> ops::Deref for ArrayVecCopy<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        self.inner.deref()
    }
}

impl<A: Array + Copy> ops::DerefMut for ArrayVecCopy<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        self.inner.deref_mut()
    }
}

/// Create an `ArrayVecCopy` from an array.
impl<A: Array + Copy> From<A> for ArrayVecCopy<A> {
    fn from(array: A) -> Self {
        ArrayVecCopy { inner: RawArrayVec::from(array) }
    }
}


/// Iterate the `ArrayVecCopy` with references to each element.
impl<'a, A: Array + Copy> IntoIterator for &'a ArrayVecCopy<A> {
    type Item = &'a A::Item;
    type IntoIter = slice::Iter<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.inner.iter() }
}

/// Iterate the `ArrayVecCopy` with mutable references to each element.
impl<'a, A: Array + Copy> IntoIterator for &'a mut ArrayVecCopy<A> {
    type Item = &'a mut A::Item;
    type IntoIter = slice::IterMut<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.inner.iter_mut() }
}

/// Iterate the `ArrayVecCopy` with each element by value.
///
/// The vector is consumed by this operation.
impl<A: Array + Copy> IntoIterator for ArrayVecCopy<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;
    fn into_iter(self) -> IntoIter<A> {
        IntoIter { index: Index::from(0), v: self }
    }
}


/// By-value iterator for `ArrayVecCopy`.
pub struct IntoIter<A: Array + Copy> {
    index: A::Index,
    v: ArrayVecCopy<A>,
}

impl<A: Array + Copy> Iterator for IntoIter<A> {
    type Item = A::Item;

    #[inline]
    fn next(&mut self) -> Option<A::Item> {
        let index = self.index.to_usize();
        if index == self.v.len() {
            None
        } else {
            unsafe {
                self.index = Index::from(index + 1);
                Some(ptr::read(self.v.get_unchecked_mut(index)))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len() - self.index.to_usize();
        (len, Some(len))
    }
}

impl<A: Array + Copy> DoubleEndedIterator for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A::Item> {
        if self.index.to_usize() == self.v.len() {
            None
        } else {
            unsafe {
                let new_len = self.v.len() - 1;
                self.v.set_len(new_len);
                Some(ptr::read(self.v.get_unchecked_mut(new_len)))
            }
        }
    }
}

impl<A: Array + Copy> ExactSizeIterator for IntoIter<A> { }

/// Extend the `ArrayVecCopy` with an iterator.
///
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array + Copy> Extend<A::Item> for ArrayVecCopy<A> {
    fn extend<T: IntoIterator<Item=A::Item>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}

/// Create an `ArrayVecCopy` from an iterator.
///
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array + Copy> iter::FromIterator<A::Item> for ArrayVecCopy<A> {
    fn from_iter<T: IntoIterator<Item=A::Item>>(iter: T) -> Self {
        ArrayVecCopy { inner: RawArrayVec::from_iter(iter) }
    }
}

impl<A: Array + Copy> Clone for ArrayVecCopy<A>
    where A::Item: Clone
{
    #[inline]
    fn clone(&self) -> Self {
        ArrayVecCopy { inner: self.inner.clone() }
    }

    #[inline]
    fn clone_from(&mut self, rhs: &Self) {
        self.inner.clone_from(&rhs.inner)
    }
}

impl<A: Array + Copy> Hash for ArrayVecCopy<A>
    where A::Item: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<A: Array + Copy> PartialEq for ArrayVecCopy<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        use std::ops::Deref;
        self.inner.eq(other.inner.deref())
    }
}

impl<A: Array + Copy> PartialEq<[A::Item]> for ArrayVecCopy<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &[A::Item]) -> bool {
        self.inner.eq(other)
    }
}

impl<A: Array + Copy> Eq for ArrayVecCopy<A> where A::Item: Eq { }

impl<A: Array + Copy> Borrow<[A::Item]> for ArrayVecCopy<A> {
    fn borrow(&self) -> &[A::Item] { self.inner.borrow() }
}

impl<A: Array + Copy> BorrowMut<[A::Item]> for ArrayVecCopy<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] { self.inner.borrow_mut() }
}

impl<A: Array + Copy> AsRef<[A::Item]> for ArrayVecCopy<A> {
    fn as_ref(&self) -> &[A::Item] { self.inner.as_ref() }
}

impl<A: Array + Copy> AsMut<[A::Item]> for ArrayVecCopy<A> {
    fn as_mut(&mut self) -> &mut [A::Item] { self.inner.as_mut() }
}

impl<A: Array + Copy> fmt::Debug for ArrayVecCopy<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.inner.fmt(f) }
}

impl<A: Array + Copy> Default for ArrayVecCopy<A> {
    fn default() -> ArrayVecCopy<A> {
        ArrayVecCopy::new()
    }
}

impl<A: Array + Copy> PartialOrd for ArrayVecCopy<A> where A::Item: PartialOrd {
    #[inline]
    fn partial_cmp(&self, other: &ArrayVecCopy<A>) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }

    #[inline] fn lt(&self, other: &Self) -> bool { self.inner.lt(&other.inner) }
    #[inline] fn le(&self, other: &Self) -> bool { self.inner.le(&other.inner) }
    #[inline] fn ge(&self, other: &Self) -> bool { self.inner.ge(&other.inner) }
    #[inline] fn gt(&self, other: &Self) -> bool { self.inner.gt(&other.inner) }
}

impl<A: Array + Copy> Ord for ArrayVecCopy<A> where A::Item: Ord {
    fn cmp(&self, other: &ArrayVecCopy<A>) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

#[cfg(feature="std")]
/// `Write` appends written data to the end of the vector.
///
/// Requires `features="std"`.
impl<A: Array<Item=u8> + Copy> io::Write for ArrayVecCopy<A> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.inner.write(data)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
