extern crate nodrop;

use nodrop::NoDrop;

use std::iter;
use std::mem;
use std::ptr;
use std::ops::{
    Deref,
    DerefMut,
};
use std::slice;

// extra traits
use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};
use std::fmt;

mod array;
mod misc;
pub use array::Array;
pub use misc::RangeArgument;
use array::Index;


unsafe fn new_array<A: Array>() -> A {
    // Note: Returning an uninitialized value here only works
    // if we can be sure the data is never used. The nullable pointer
    // inside enum optimization conflicts with this this for example,
    // so we need to be extra careful. See `Flag` enum.
    mem::uninitialized()
}

/// A vector with a fixed capacity.
///
/// The **ArrayVec** is a vector backed by a fixed size array. It keeps track of
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
    xs: NoDrop<A>,
    len: A::Index,
}

impl<A: Array> Drop for ArrayVec<A> {
    fn drop(&mut self) {
        // clear all elements, then NoDrop inhibits drop of inner array
        while let Some(_) = self.pop() { }
    }
}

impl<A: Array> ArrayVec<A> {
    /// Create a new empty **ArrayVec**.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// ## Examples
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
        unsafe {
            ArrayVec { xs: NoDrop::new(new_array()), len: Index::zero() }
        }
    }

    /// Return the number of elements in the **ArrayVec**.
    ///
    /// ## Examples
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut array = ArrayVec::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize { self.len.to_usize() }

    unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= self.capacity());
        self.len = Index::from(length);
    }

    /// Return the capacity of the **ArrayVec**.
    ///
    /// ## Examples
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let array = ArrayVec::from([1, 2, 3]);
    /// assert_eq!(array.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { A::capacity() }

    /// Remove all elements in the vector.
    pub fn clear(&mut self) {
        while let Some(_) = self.pop() { }
    }

    /// Push **element** to the end of the vector.
    ///
    /// Return **None** if the push succeeds, or and return **Some(** *element* **)**
    /// if the vector is full.
    ///
    /// ## Examples
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
    pub fn push(&mut self, element: A::Item) -> Option<A::Item> {
        if self.len() < A::capacity() {
            let len = self.len();
            unsafe {
                ptr::write(self.get_unchecked_mut(len), element);
                self.set_len(len + 1);
            }
            None
        } else {
            Some(element)
        }
    }

    /// Remove the last element in the vector.
    ///
    /// Return **Some(** *element* **)** if the vector is non-empty, else **None**.
    ///
    /// ## Examples
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

    /// Remove the element at **index** and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return **Some(** *element* **)** if the index is in bounds, else **None**.
    ///
    /// ## Examples
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
    pub fn swap_remove(&mut self, index: usize) -> Option<A::Item> {
        let len = self.len();
        if index >= len {
            return None
        }
        self.swap(index, len - 1);
        self.pop()
    }

    /// Remove the element at **index** and shift down the following elements.
    ///
    /// Return **Some(** *element* **)** if the index is in bounds, else **None**.
    ///
    /// ## Examples
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
    pub fn remove(&mut self, index: usize) -> Option<A::Item> {
        if index >= self.len() {
            None
        } else {
            self.drain(index..index + 1).next()
        }
    }

    /// Insert **element** in position **index**.
    ///
    /// Shift up all elements after **index**. If any is pushed out, it is returned.
    ///
    /// Return None if no element is shifted out.
    ///
    /// ## Examples
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
    pub fn insert(&mut self, index: usize, element: A::Item) -> Option<A::Item> {
        if index >= self.capacity() {
            return Some(element);
        }
        let mut ret = None;
        let old_len = self.len();
        if old_len == self.capacity() {
            ret = self.pop();
        }
        let len = self.len();

        // follows is just like Vec<T>
        unsafe { // infallible
            // The spot to put the new value
            {
                let p = self.as_mut_ptr().offset(index as isize);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(&*p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(&mut *p, element);
            }
            self.set_len(len + 1);
        }
        ret
    }

    /// Create a draining iterator that removes the specified range in the vector
    /// and yields the removed items from start to end. The element range is
    /// removed even if the iterator is not consumed until the end.
    ///
    /// Note: It is unspecified how many elements are removed from the vector,
    /// if the `Drain` value is leaked.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use arrayvec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from([1, 2, 3]);
    /// let u: Vec<_> = v.drain(0..2).collect();
    /// assert_eq!(&v[..], &[3]);
    /// assert_eq!(&u[..], &[1, 2]);
    /// ```
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
}

impl<A: Array> Deref for ArrayVec<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        unsafe {
            slice::from_raw_parts(self.xs.as_ptr(), self.len())
        }
    }
}

impl<A: Array> DerefMut for ArrayVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts_mut(self.xs.as_mut_ptr(), len)
        }
    }
}

/// Create an **ArrayVec** from an array.
///
/// ## Examples
/// ```
/// use arrayvec::ArrayVec;
///
/// let mut array = ArrayVec::from([1, 2, 3]);
/// assert_eq!(array.len(), 3);
/// assert_eq!(array.capacity(), 3);
/// ```
impl<A: Array> From<A> for ArrayVec<A> {
    fn from(array: A) -> Self {
        ArrayVec { xs: NoDrop::new(array), len: Index::from(A::capacity()) }
    }
}


/// Iterate the **ArrayVec** with references to each element.
///
/// ## Examples
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
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// Iterate the **ArrayVec** with mutable references to each element.
///
/// ## Examples
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
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

/// Iterate the **ArrayVec** with each element by value.
///
/// The vector is consumed by this operation.
///
/// ## Examples
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
        IntoIter { index: Index::zero(), v: self, }
    }
}


/// By-value iterator for **ArrayVec**.
pub struct IntoIter<A: Array> {
    index: A::Index,
    v: ArrayVec<A>,
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;

    #[inline]
    fn next(&mut self) -> Option<A::Item> {
        if self.index == self.v.len {
            None
        } else {
            unsafe {
                let index = self.index.to_usize();
                self.index = Index::from(index + 1);
                Some(ptr::read(self.v.get_unchecked_mut(index)))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len() - self.index.to_usize();
        (len, Some(len))
    }
}

impl<A: Array> DoubleEndedIterator for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A::Item> {
        if self.index == self.v.len {
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
        // exhaust iterator and clear the vector
        while let Some(_) = self.next() { }
        unsafe {
            self.v.set_len(0);
        }
    }
}

/// A draining iterator for **ArrayVec**.
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
    vec: *mut ArrayVec<A>,
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




/// Extend the **ArrayVec** with an iterator.
/// 
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array> Extend<A::Item> for ArrayVec<A> {
    fn extend<T: IntoIterator<Item=A::Item>>(&mut self, iter: T) {
        let take = self.capacity() - self.len();
        for elt in iter.into_iter().take(take) {
            self.push(elt);
        }
    }
}

/// Create an **ArrayVec** from an iterator.
/// 
/// Does not extract more items than there is space for. No error
/// occurs if there are more iterator elements.
impl<A: Array> iter::FromIterator<A::Item> for ArrayVec<A> {
    fn from_iter<T: IntoIterator<Item=A::Item>>(iter: T) -> Self {
        let mut array = ArrayVec::new();
        array.extend(iter);
        array
    }
}

impl<A: Array> Clone for ArrayVec<A>
    where A::Item: Clone
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<A: Array> Hash for ArrayVec<A>
    where A::Item: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<A: Array> PartialEq for ArrayVec<A>
    where A::Item: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<A: Array> Eq for ArrayVec<A> where A::Item: Eq { }

impl<A: Array> Borrow<[A::Item]> for ArrayVec<A> {
    fn borrow(&self) -> &[A::Item] { self }
}

impl<A: Array> BorrowMut<[A::Item]> for ArrayVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] { self }
}

impl<A: Array> AsRef<[A::Item]> for ArrayVec<A> {
    fn as_ref(&self) -> &[A::Item] { self }
}

impl<A: Array> AsMut<[A::Item]> for ArrayVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] { self }
}

impl<A: Array> fmt::Debug for ArrayVec<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

#[test]
fn test_simple() {
    use std::ops::Add;

    let mut vec: ArrayVec<[Vec<i32>; 3]> = ArrayVec::new();

    vec.push(vec![1,2,3,4]);
    vec.push(vec![3]);
    vec.push(vec![-1, 90, -2]);

    for elt in &vec {
        println!("{:?}", elt);
    }

    println!("{:?}", vec);

    let sum = vec.iter().map(|x| x.iter().fold(0, Add::add)).fold(0, Add::add);
    assert_eq!(sum, 13 + 87);
    let sum_len = vec.into_iter().map(|x| x.len()).fold(0, Add::add);
    assert_eq!(sum_len, 8);
}

#[test]
fn test_u16_index() {
    const N: usize = 4096;
    let mut vec: ArrayVec<[_; N]> = ArrayVec::new();
    for _ in 0..N {
        assert!(vec.push(1u8).is_none());
    }
    assert!(vec.push(0).is_some());
    assert_eq!(vec.len(), N);
}

#[test]
fn test_iter() {
    let mut iter = ArrayVec::from([1, 2, 3]).into_iter();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(2));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next_back(), None);
}

#[test]
fn test_drop() {
    use std::cell::Cell;

    let flag = &Cell::new(0);

    struct Bump<'a>(&'a Cell<i32>);

    impl<'a> Drop for Bump<'a> {
        fn drop(&mut self) {
            let n = self.0.get();
            self.0.set(n + 1);
        }
    }

    {
        let mut array = ArrayVec::<[Bump; 128]>::new();
        array.push(Bump(flag));
        array.push(Bump(flag));
    }
    assert_eq!(flag.get(), 2);

    // test something with the nullable pointer optimization
    flag.set(0);

    {
        let mut array = ArrayVec::<[_; 3]>::new();
        array.push(vec![Bump(flag)]);
        array.push(vec![Bump(flag), Bump(flag)]);
        array.push(vec![]);
        array.push(vec![Bump(flag)]);
        assert_eq!(flag.get(), 1);
        drop(array.pop());
        assert_eq!(flag.get(), 1);
        drop(array.pop());
        assert_eq!(flag.get(), 3);
    }

    assert_eq!(flag.get(), 4);
}

#[test]
fn test_extend() {
    let mut range = 0..10;

    let mut array: ArrayVec<[_; 5]> = range.by_ref().collect();
    assert_eq!(&array[..], &[0, 1, 2, 3, 4]);
    assert_eq!(range.next(), Some(5));

    array.extend(range.by_ref());
    assert_eq!(range.next(), Some(6));

    let mut array: ArrayVec<[_; 10]> = (0..3).collect();
    assert_eq!(&array[..], &[0, 1, 2]);
    array.extend(3..5);
    assert_eq!(&array[..], &[0, 1, 2, 3, 4]);
}

#[test]
fn test_is_send_sync() {
    let data = ArrayVec::<[Vec<i32>; 5]>::new();
    &data as &Send;
    &data as &Sync;
}

#[test]
fn test_compact_size() {
    // Future rust will kill these drop flags!
    // 4 elements size + 1 len + 1 enum tag + [1 drop flag] + [1 drop flag nodrop]
    type ByteArray = ArrayVec<[u8; 4]>;
    println!("{}", mem::size_of::<ByteArray>());
    assert!(mem::size_of::<ByteArray>() <= 8);

    // 12 element size + 1 len + 1 drop flag + 2 padding + 1 enum tag + 3 padding
    type QuadArray = ArrayVec<[u32; 3]>;
    println!("{}", mem::size_of::<QuadArray>());
    assert!(mem::size_of::<QuadArray>() <= 24);
}

#[test]
fn test_drain() {
    let mut v = ArrayVec::from([0; 8]);
    v.pop();
    v.drain(0..7);
    assert_eq!(&v[..], &[]);

    v.extend(0..);
    v.drain(1..4);
    assert_eq!(&v[..], &[0, 4, 5, 6, 7]);
    let u: ArrayVec<[_; 3]> = v.drain(1..4).rev().collect();
    assert_eq!(&u[..], &[6, 5, 4]);
    assert_eq!(&v[..], &[0, 7]);
    v.drain(..);
    assert_eq!(&v[..], &[]);
}

#[test]
#[should_panic]
fn test_drain_oob() {
    let mut v = ArrayVec::from([0; 8]);
    v.pop();
    v.drain(0..8);
}

#[test]
fn test_insert() {
    let mut v = ArrayVec::from([]);
    assert_eq!(v.push(1), Some(1));
    assert_eq!(v.insert(0, 1), Some(1));

    let mut v = ArrayVec::<[_; 3]>::new();
    v.insert(0, 0);
    v.insert(1, 1);
    v.insert(2, 2);
    v.insert(3, 3);
    assert_eq!(&v[..], &[0, 1, 2]);
    v.insert(1, 9);
    assert_eq!(&v[..], &[0, 9, 1]);

    let mut v = ArrayVec::from([2]);
    assert_eq!(v.insert(1, 1), Some(1));
    assert_eq!(v.insert(2, 1), Some(1));
}
