
use std::ops::{Deref, DerefMut};
use odds::debug_assert_unreachable;

/// repr(u8) - Make sure the non-nullable pointer optimization does not occur!
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Uninit<T> {
    Alive(T),
    #[allow(dead_code)]
    Dropped,
}

pub unsafe fn new<T>(value: T) -> Uninit<T> {
    Uninit::Alive(value)
}

impl<T> Deref for Uninit<T> {
    type Target = T;

    // Use type invariant, always Uninit::Alive.
    #[inline]
    fn deref(&self) -> &T {
        match *self {
            Uninit::Alive(ref inner) => inner,
            _ => unsafe { debug_assert_unreachable() }
        }
    }
}

impl<T> DerefMut for Uninit<T> {
    // Use type invariant, always Uninit::Alive.
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            Uninit::Alive(ref mut inner) => inner,
            _ => unsafe { debug_assert_unreachable() }
        }
    }
}
