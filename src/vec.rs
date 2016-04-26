use RangeArgument;
use array::Array;
use raw::RawArrayVec;
use raw::Drain;
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

use nodrop::NoDrop;

use array::Index;

/// A vector with a fixed capacity.
///
/// The `ArrayVec` is a vector backed by a fixed size array. It keeps track of
/// the number of initialized elements.
///
/// The vector is a contiguous value that you can store directly on the stack
/// if needed.
///
/// It offers a simple API but also dereferences to a slice, so
/// that the full slice API is available.
///
/// ArrayVec can be converted into a by value iterator.
pub struct ArrayVec<A: Array> {
    inner: NoDrop<RawArrayVec<A>>,
}

impl<A: Array> Drop for ArrayVec<A> {
    fn drop(&mut self) {
        self.clear();

        // NoDrop inhibits array's drop
        // panic safety: NoDrop::drop will trigger on panic, so the inner
        // array will not drop even after panic.
    }
}

impl<A: Array> ArrayVec<A> {
    /// Create a new empty `ArrayVec`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::<[_; 16]>::new();
    /// array.push(1);
    /// array.push(2);
    /// assert_eq!(&array[..], &[1, 2]);
    /// assert_eq!(array.capacity(), 16);
    /// ```
    pub fn new() -> ArrayVec<A> {
        ArrayVec {
            inner: NoDrop::new(RawArrayVec::new())
        }
    }

    /// Return the number of elements in the `ArrayVec`.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize { self.inner.len() }

    /// Return the capacity of the `ArrayVec`.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let array = ArrayVec::from([1, 2, 3]);
    /// assert_eq!(array.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { self.inner.capacity() }

    /// Return if the `ArrayVec` is completely filled.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::<[_; 1]>::new();
    /// assert!(!array.is_full());
    /// array.push(1);
    /// assert!(array.is_full());
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool { self.inner.is_full() }

    /// Push `element` to the end of the vector.
    ///
    /// Return `None` if the push succeeds, or and return `Some(` *element* `)`
    /// if the vector is full.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::<[_; 2]>::new();
    ///
    /// array.push(1);
    /// array.push(2);
    /// let overflow = array.push(3);
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    /// assert_eq!(overflow, Some(3));
    /// ```
    #[inline]
    pub fn push(&mut self, element: A::Item) -> Option<A::Item> {
        self.inner.push(element).err().map(|e| e.element())
    }

    /// Insert `element` in position `index`.
    ///
    /// Shift up all elements after `index`. If any is pushed out, it is returned.
    ///
    /// Return `None` if no element is shifted out.
    ///
    /// `index` must be <= `self.len()` and < `self.capacity()`. Note that any
    /// out of bounds index insert results in the element being "shifted out"
    /// and returned directly.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::<[_; 2]>::new();
    ///
    /// assert_eq!(array.insert(0, "x"), None);
    /// assert_eq!(array.insert(0, "y"), None);
    /// assert_eq!(array.insert(0, "z"), Some("x"));
    /// assert_eq!(array.insert(1, "w"), Some("y"));
    /// assert_eq!(&array[..], &["z", "w"]);
    ///
    /// ```
    #[inline]
    pub fn insert(&mut self, index: usize, element: A::Item) -> Option<A::Item> {
        if index > self.len() {
            return Some(element);
        }
        self.inner.insert(index, element).err().map(|e| e.element())
    }

    /// Remove the last element in the vector.
    ///
    /// Return `Some(` *element* `)` if the vector is non-empty, else `None`.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::<[_; 2]>::new();
    ///
    /// array.push(1);
    ///
    /// assert_eq!(array.pop(), Some(1));
    /// assert_eq!(array.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<A::Item> {
        self.inner.pop()
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_remove(0), Some(1));
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_remove(10), None);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> Option<A::Item> {
        self.inner.swap_remove(index)
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::from([1, 2, 3]);
    ///
    /// assert_eq!(array.remove(0), Some(1));
    /// assert_eq!(&array[..], &[2, 3]);
    ///
    /// assert_eq!(array.remove(10), None);
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<A::Item> {
        self.inner.remove(index)
    }

    /// Remove all elements in the vector.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns false.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::from([1, 2, 3, 4]);
    /// array.retain(|x| *x & 1 != 0 );
    /// assert_eq!(&array[..], &[1, 3]);
    /// ```
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
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from([1, 2, 3]);
    /// let u: Vec<_> = v.drain(0..2).collect();
    /// assert_eq!(&v[..], &[3]);
    /// assert_eq!(&u[..], &[1, 2]);
    /// ```
    #[inline]
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
    #[inline]
    pub fn into_inner(self) -> Result<A, Self> {
        let inner;
        unsafe {
            inner = ptr::read(&self.inner);
            mem::forget(self);
        }
        inner.into_inner().into_inner().map_err(|e| ArrayVec { inner: NoDrop::new(e) })
    }

    /// Dispose of `self` without the overwriting that is needed in Drop.
    #[inline]
    pub fn dispose(mut self) {
        self.clear();
        mem::forget(self);
    }

    /// Return a slice containing all elements of the vector.
    #[inline]
    pub fn as_slice(&self) -> &[A::Item] {
        self.inner.as_slice()
    }

    /// Return a mutable slice containing all elements of the vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [A::Item] {
        self.inner.as_mut_slice()
    }
}

impl<A: Array> Deref for ArrayVec<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        self.inner.deref()
    }
}

impl<A: Array> DerefMut for ArrayVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        self.inner.deref_mut()
    }
}

/// Create an `ArrayVec` from an array.
///
/// ```
/// use arrayvec::ArrayVec;
///
/// let mut array = ArrayVec::from([1, 2, 3]);
/// assert_eq!(array.len(), 3);
/// assert_eq!(array.capacity(), 3);
/// ```
impl<A: Array> From<A> for ArrayVec<A> {
    fn from(array: A) -> Self {
        ArrayVec { inner: NoDrop::new(RawArrayVec::from(array)) }
    }
}


/// Iterate the `ArrayVec` with references to each element.
///
/// ```
/// use arrayvec::ArrayVec;
///
/// let array = ArrayVec::from([1, 2, 3]);
///
/// for elt in &array {
///     // ...
/// }
/// ```
impl<'a, A: Array> IntoIterator for &'a ArrayVec<A> {
    type Item = &'a A::Item;
    type IntoIter = slice::Iter<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.inner.iter() }
}

/// Iterate the `ArrayVec` with mutable references to each element.
///
/// ```
/// use arrayvec::ArrayVec;
///
/// let mut array = ArrayVec::from([1, 2, 3]);
///
/// for elt in &mut array {
///     // ...
/// }
/// ```
impl<'a, A: Array> IntoIterator for &'a mut ArrayVec<A> {
    type Item = &'a mut A::Item;
    type IntoIter = slice::IterMut<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.inner.iter_mut() }
}

/// Iterate the `ArrayVec` with each element by value.
///
/// The vector is consumed by this operation.
///
/// ```
/// use arrayvec::ArrayVec;
///
/// for elt in ArrayVec::from([1, 2, 3]) {
///     // ...
/// }
/// ```
impl<A: Array> IntoIterator for ArrayVec<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;
    fn into_iter(self) -> IntoIter<A> {
        IntoIter { index: Index::from(0), v: self }
    }
}


/// By-value iterator for `ArrayVec`.
pub struct IntoIter<A: Array> {
    index: A::Index,
    v: ArrayVec<A>,
}

impl<A: Array> Iterator for IntoIter<A> {
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

impl<A: Array> DoubleEndedIterator for IntoIter<A> {
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

impl<A: Array> ExactSizeIterator for IntoIter<A> { }

impl<A: Array> Drop for IntoIter<A> {
    fn drop(&mut self) {
        // panic safety: Set length to 0 before dropping elements.
        let index = self.index.to_usize();
        let len = self.v.len();
        unsafe {
            self.v.set_len(0);
            let elements = slice::from_raw_parts(self.v.get_unchecked_mut(index),
                                                 len - index);
            for elt in elements {
                ptr::read(elt);
            }
        }
    }
}

/// Extend the `ArrayVec` with an iterator.
/// 
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array> Extend<A::Item> for ArrayVec<A> {
    fn extend<T: IntoIterator<Item=A::Item>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}

/// Create an `ArrayVec` from an iterator.
/// 
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array> iter::FromIterator<A::Item> for ArrayVec<A> {
    fn from_iter<T: IntoIterator<Item=A::Item>>(iter: T) -> Self {
        // Cannot use `RawArrayVec::from_iter` because it's not wrapped in
        // `ArrayVec` immediately which makes it panic-unsafe.
        let mut array = ArrayVec::new();
        array.extend(iter);
        array
    }
}

impl<A: Array> Clone for ArrayVec<A>
    where A::Item: Clone
{
    #[inline]
    fn clone(&self) -> Self {
        // Can't use `RawArrayVec::clone` here because it's not wrapped in
        // `ArrayVec`, see above.
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.inner.clone_from(&rhs.inner)
    }
}

impl<A: Array> Hash for ArrayVec<A>
    where A::Item: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<A: Array> PartialEq for ArrayVec<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        use std::ops::Deref;
        self.inner.eq(other.inner.deref())
    }
}

impl<A: Array> PartialEq<[A::Item]> for ArrayVec<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &[A::Item]) -> bool {
        self.inner.eq(other)
    }
}

impl<A: Array> Eq for ArrayVec<A> where A::Item: Eq { }

impl<A: Array> Borrow<[A::Item]> for ArrayVec<A> {
    fn borrow(&self) -> &[A::Item] { self.inner.borrow() }
}

impl<A: Array> BorrowMut<[A::Item]> for ArrayVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] { self.inner.borrow_mut() }
}

impl<A: Array> AsRef<[A::Item]> for ArrayVec<A> {
    fn as_ref(&self) -> &[A::Item] { self.inner.as_ref() }
}

impl<A: Array> AsMut<[A::Item]> for ArrayVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] { self.inner.as_mut() }
}

impl<A: Array> fmt::Debug for ArrayVec<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.inner.fmt(f) }
}

impl<A: Array> Default for ArrayVec<A> {
    fn default() -> ArrayVec<A> {
        ArrayVec { inner: NoDrop::new(Default::default()) }
    }
}

impl<A: Array> PartialOrd for ArrayVec<A> where A::Item: PartialOrd {
    #[inline]
    fn partial_cmp(&self, other: &ArrayVec<A>) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }

    #[inline] fn lt(&self, other: &Self) -> bool { self.inner.lt(&other.inner) }
    #[inline] fn le(&self, other: &Self) -> bool { self.inner.le(&other.inner) }
    #[inline] fn ge(&self, other: &Self) -> bool { self.inner.ge(&other.inner) }
    #[inline] fn gt(&self, other: &Self) -> bool { self.inner.gt(&other.inner) }
}

impl<A: Array> Ord for ArrayVec<A> where A::Item: Ord {
    #[inline]
    fn cmp(&self, other: &ArrayVec<A>) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

#[cfg(feature="std")]
/// `Write` appends written data to the end of the vector.
///
/// Requires `features="std"`.
impl<A: Array<Item=u8>> io::Write for ArrayVec<A> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.inner.write(data)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
