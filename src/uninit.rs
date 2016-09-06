
use std::ops::{Deref, DerefMut};

/// repr(u8) - Make sure the non-nullable pointer optimization does not occur!
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Uninit<T> {
    Alive(T),
    #[allow(dead_code)]
    Void(Void),
}

#[derive(Copy, Clone)]
pub enum Void { }

pub fn new<T>(value: T) -> Uninit<T> {
    Uninit::Alive(value)
}

impl<T> Deref for Uninit<T> {
    type Target = T;

    // Use type invariant, always Uninit::Alive.
    #[inline]
    fn deref(&self) -> &T {
        match *self {
            Uninit::Alive(ref inner) => inner,
            Uninit::Void(ref inner) => match *inner { /* unreachable */ }
        }
    }
}

impl<T> DerefMut for Uninit<T> {
    // Use type invariant, always Uninit::Alive.
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            Uninit::Alive(ref mut inner) => inner,
            Uninit::Void(ref inner) => match *inner { /* unreachable */ }
        }
    }
}
