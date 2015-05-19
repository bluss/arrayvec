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

#[derive(Debug)]
struct Dud(i32);

impl Drop for Dud {
    fn drop(&mut self) {
        let Dud(ref i) = *self;
        println!("Drop Dud({})", *i);
    }
}

/// Trait for fixed size arrays.
pub unsafe trait Array {
    type Item;
    unsafe fn new() -> Self;
    fn as_ptr(&self) -> *const Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    fn capacity() -> usize;
}

macro_rules! fix_array_impl {
    ($len:expr ) => (
        unsafe impl<T> Array for [T; $len] {
            type Item = T;
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
                          32, 48, 64, 96, 128,);

/// A vector with a fixed capacity.
///
/// The **ArrayVec** is backed by a fixed size array and keeps track of
/// the number of initialized elements.
///
/// The ArrayVec is a congiguous value that you can store directly on the stack
/// if needed.
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
    pub fn new() -> ArrayVec<A> {
        unsafe {
            ArrayVec { xs: Flag::Alive(Array::new()), len: 0 }
        }
    }

    pub fn from(array: A) -> ArrayVec<A> {
        ArrayVec { xs: Flag::Alive(array), len: A::capacity() as u8 }
    }

    #[inline]
    pub fn capacity(&self) -> usize { A::capacity() }

    #[inline]
    fn array(&self) -> &A {
        match self.xs {
            Flag::Alive(ref xs) => xs,
            _ => unreachable!(),
            //_ => std::intrinsics::unreachable(),
        }
    }

    #[inline]
    fn array_mut(&mut self) -> &mut A {
        // FIXME: Optimize this, we know it's always Some.
        match self.xs {
            Flag::Alive(ref mut xs) => xs,
            _ => unreachable!(),
            //_ => std::intrinsics::unreachable(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize { self.len as usize }

    pub fn push(&mut self, elt: A::Item) -> Option<A::Item> {
        if self.len() < A::capacity() {
            unsafe {
                let len = self.len();
                ptr::write(self.get_unchecked_mut(len), elt);
            }
            self.len += 1;
            None
        } else {
            Some(elt)
        }
    }

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
            slice::from_raw_parts(self.array().as_ptr(), self.len())
        }
    }
}

impl<A: Array> DerefMut for ArrayVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts_mut(self.array_mut().as_mut_ptr(), len)
        }
    }
}

impl<'a, A: Array> IntoIterator for &'a ArrayVec<A> {
    type Item = &'a A::Item;
    type IntoIter = slice::Iter<'a, A::Item>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

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

#[test]
fn test1() {
    let mut vec: ArrayVec<[Vec<i32>; 3]> = ArrayVec::new();

    vec.push(vec![1,2,4,5]);
    vec.push(vec![3]);
    vec.push(vec![97,98,92]);

    for elt in vec {
        println!("{:?}", elt);
    }
}

fn main() {
    let mut v = ArrayVec::from([1, 2, 3]);
    v.push(4);
    v.push(5);
    println!("{:?}", v.pop());
    println!("{:?}", v.pop());
    println!("{:?}", &*v);
    v.pop();
    v.pop();
    v.push(8);
    println!("{:?}", &*v);

    let mut u: ArrayVec<[_; 3]> = ArrayVec::new();

    u.push(vec![1,2,4,5]);
    u.push(vec![3]);
    u.push(vec![97,98,92]);

    {
        let slc: &[_] = &u;
        println!("{:?}", slc);
    }
    println!("{:?}", u.pop());
    println!("{:?}", u.pop());
    println!("{:?}", u.len());
    println!("{:?}", u[0]);


    let mut v: ArrayVec<[Dud; 2]> = ArrayVec::new();
    v.push(Dud(1));
    v.push(Dud(2));
    v.pop();
    v.pop();
    v.pop();
    v.push(Dud(3));
    v.pop();
    v.push(Dud(4));
    v.push(Dud(5));
    v.push(Dud(6));
    //v.pop();

    println!("v: {:?}", &*v);

    for elt in &v { // slice iter
        println!("Slice Iter: {:?}", elt);
    }

    for elt in v {
        println!("Iter: {:?}", elt);
        //break;
    }

    for elt in ArrayVec::from(["a".to_string(), "b".to_string()]).into_iter() {
        println!("Iter: {:?}", elt);
    }
}
