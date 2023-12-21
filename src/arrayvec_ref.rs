use std::ops::{Deref, DerefMut};
use crate::ArrayVec;

// ArrayVecRef is useless because we have Deref<[T}> already

/// A mutable reference to an ArrayVec
/// It gives you all the access of a mutable reference to an ArrayVec
/// instead only a few methods exposed by ArrayVecImpl
pub struct ArrayVecRefMut<'a, T> {
    vec: &'a mut ArrayVec<T, 0>,
}

impl<'a, T> ArrayVecRefMut<'a, T> {
    pub fn new<const CAP: usize>(vec: &'a mut ArrayVec<T, CAP>) -> Self {
        unsafe {
            Self { vec: std::mem::transmute(vec) }
        }
    }
}


impl<'a, T> Deref for ArrayVecRefMut<'a, T> {
    type Target = ArrayVec<T, 0>;
    fn deref(&self) -> &Self::Target {
        self.vec
    }
}

impl<'a, T> DerefMut for ArrayVecRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}