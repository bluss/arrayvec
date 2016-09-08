//!
//! The **nodrop** crate has the following cargo feature flags:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Requires Rust 1.6 *to disable*
//!   - Use libstd
//! - `use_needs_drop`
//!   - Optional
//!   - Requires nightly channel.
//!   - Use `needs_drop` to skip overwriting if not necessary
//! - `use_union`
//!   - Optional
//!   - Requires nightly channel
//!   - Using untagged union, finally we have an implementation of `NoDrop` without hacks,
//!     for example the fact that `NoDrop<T>` never has a destructor anymore.
//!

#![feature(untagged_unions)]

#![cfg_attr(not(test), no_std)]
#[cfg(not(test))]
extern crate core as std;

use std::ops::{Deref, DerefMut};

#[allow(unions_with_drop_fields)]
#[derive(Copy)]
union UnionFlag<T> {
    value: T,
}

impl<T: Clone> Clone for UnionFlag<T> {
    fn clone(&self) -> Self {
        unsafe {
            UnionFlag { value: self.value.clone() }
        }
    }
}

#[derive(Copy, Clone)]
pub struct NoDrop<T>(UnionFlag<T>);

impl<T> NoDrop<T> {
    /// Create a new **NoDrop**.
    #[inline]
    pub fn new(value: T) -> Self {
        NoDrop(UnionFlag { value: value })
    }

    /// Extract the inner value.
    ///
    /// Once extracted, the value can of course drop again.
    #[inline]
    pub fn into_inner(self) -> T {
        unsafe {
            self.0.value
        }
    }
}

impl<T> Deref for NoDrop<T> {
    type Target = T;

    // Use type invariant, always initialized
    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            &self.0.value
        }
    }
}

impl<T> DerefMut for NoDrop<T> {
    // Use type invariant, always initialized
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut self.0.value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDrop;
    use std::mem;

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
            let _ = NoDrop::new([Bump(flag), Bump(flag)]);
        }
        assert_eq!(flag.get(), 0);

        // test something with the nullable pointer optimization
        flag.set(0);

        {
            let mut array = NoDrop::new(Vec::new());
            array.push(vec![Bump(flag)]);
            array.push(vec![Bump(flag), Bump(flag)]);
            array.push(vec![]);
            array.push(vec![Bump(flag)]);
            drop(array.pop());
            assert_eq!(flag.get(), 1);
            drop(array.pop());
            assert_eq!(flag.get(), 1);
            drop(array.pop());
            assert_eq!(flag.get(), 3);
        }

        // last one didn't drop.
        assert_eq!(flag.get(), 3);

        flag.set(0);
        {
            let array = NoDrop::new(Bump(flag));
            array.into_inner();
            assert_eq!(flag.get(), 1);
        }
        assert_eq!(flag.get(), 1);
    }

    #[test]
    fn test_size_of() {
        assert!(mem::size_of::<NoDrop<&i32>>() == mem::size_of::<&i32>());
        assert!(mem::size_of::<NoDrop<Vec<i32>>>() == mem::size_of::<Vec<i32>>());
        // No non-nullable pointer optimization!
        assert!(mem::size_of::<Option<NoDrop<&i32>>>() > mem::size_of::<NoDrop<&i32>>());
    }
}
