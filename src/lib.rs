use std::iter;
use std::mem;
use std::ptr;
use std::ops::{
    Deref,
    DerefMut,
};
use std::slice;
use std::convert::From;

/// Make sure the non-nullable pointer optimization does not occur!
enum Flag<T> {
    Alive(T),
    Dropped,
    _Unused,
}

/// Trait for fixed size arrays.
pub unsafe trait Array {
    /// The array's element type
    type Item;
    #[doc(hidden)]
    unsafe fn new() -> Self;
    #[doc(hidden)]
    fn as_ptr(&self) -> *const Self::Item;
    #[doc(hidden)]
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    #[doc(hidden)]
    fn capacity() -> usize;
}

macro_rules! fix_array_impl {
    ($len:expr ) => (
        unsafe impl<T> Array for [T; $len] {
            type Item = T;
            /// Note: Returnin an uninitialized value here only works
            /// if we can be sure the data is never used. The nullable pointer
            /// inside enum optimization conflicts with this this for example,
            /// so we need to be extra careful. See `Flag` enum.
            unsafe fn new() -> [T; $len] { mem::uninitialized() }
            #[inline]
            fn as_ptr(&self) -> *const T { self as *const _ as *const _ }
            fn as_mut_ptr(&mut self) -> *mut T { self as *mut _ as *mut _}
            #[inline]
            fn capacity() -> usize { $len }
        }
    )
}

macro_rules! fix_array_impl_recursive {
    () => ();
    ($len:expr, $($more:expr,)*) => (
        fix_array_impl!($len);
        fix_array_impl_recursive!($($more,)*);
    );
}

fix_array_impl_recursive!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                          16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                          32, 40, 48, 56, 64, 72, 96, 128, 160, 192, 224,);

/// A vector with a fixed capacity.
///
/// The **ArrayVec** is a vector backed by a fixed size array and keeps track of
/// the number of initialized elements.
///
/// The vector is a contiguous value that you can store directly on the stack
/// if needed.
///
/// It offers a simple API of *.push()* and *.pop()* but also dereferences to a slice, so
/// that the full slice API is available.
///
/// The vector also implements a by value iterator.
pub struct ArrayVec<A: Array> {
    len: u8,
    xs: Flag<A>,
}

impl<A: Array> Drop for ArrayVec<A> {
    fn drop(&mut self) {
        // clear all elements, then inhibit drop of inner array
        while let Some(_) = self.pop() { }
        unsafe {
            ptr::write(&mut self.xs, Flag::Dropped);
        }
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
            ArrayVec { xs: Flag::Alive(Array::new()), len: 0 }
        }
    }

    #[inline]
    fn inner_ref(&self) -> &A {
        match self.xs {
            Flag::Alive(ref xs) => xs,
            _ => unreachable!(),
            //_ => std::intrinsics::unreachable(),
        }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut A {
        // FIXME: Optimize this, we know it's always Some.
        match self.xs {
            Flag::Alive(ref mut xs) => xs,
            _ => unreachable!(),
            //_ => std::intrinsics::unreachable(),
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
    pub fn len(&self) -> usize { self.len as usize }

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
            unsafe {
                let len = self.len();
                ptr::write(self.get_unchecked_mut(len), element);
            }
            self.len += 1;
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
        if self.len == 0 {
            return None
        }
        unsafe {
            self.len -= 1;
            let len = self.len();
            Some(ptr::read(self.get_unchecked_mut(len)))
        }
    }
}

impl<A: Array> Deref for ArrayVec<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        unsafe {
            slice::from_raw_parts(self.inner_ref().as_ptr(), self.len())
        }
    }
}

impl<A: Array> DerefMut for ArrayVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts_mut(self.inner_mut().as_mut_ptr(), len)
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
        ArrayVec { xs: Flag::Alive(array), len: A::capacity() as u8 }
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
        IntoIter { index: 0, v: self, }
    }
}


/// By-value iterator for ArrayVec.
pub struct IntoIter<A: Array> {
    index: u8,
    v: ArrayVec<A>,
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<A::Item> {
        if self.index == self.v.len {
            None
        } else {
            unsafe {
                let ptr = self.v.get_unchecked_mut(self.index as usize);
                let elt = ptr::read(ptr);
                self.index += 1;
                Some(elt)
            }
        }
    }
}

impl<A: Array> Drop for IntoIter<A> {
    fn drop(&mut self) {
        // exhaust iterator and clear the vector
        while let Some(_) = self.next() { }
        self.v.len = 0;
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

    let sum = vec.iter().map(|x| x.iter().fold(0, Add::add)).fold(0, Add::add);
    assert_eq!(sum, 13 + 87);
    let sum_len = vec.into_iter().map(|x| x.len()).fold(0, Add::add);
    assert_eq!(sum_len, 8);
}

#[test]
fn test_drop() {
    use std::rc::Rc;
    use std::cell::Cell;

    let flag = Rc::new(Cell::new(0));

    struct Foo(Rc<Cell<i32>>);

    impl Drop for Foo {
        fn drop(&mut self) {
            let n = self.0.get();
            self.0.set(n + 1);
        }
    }

    {
        let mut array = ArrayVec::<[Foo; 128]>::new();
        array.push(Foo(flag.clone()));
        array.push(Foo(flag.clone()));
    }
    assert_eq!(flag.get(), 2);

    // test something with the nullable pointer optimization
    flag.set(0);

    {
        let mut array = ArrayVec::<[_; 3]>::new();
        array.push(vec![Foo(flag.clone())]);
        array.push(vec![Foo(flag.clone()), Foo(flag.clone())]);
        array.push(vec![]);
        array.push(vec![Foo(flag.clone())]);
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
