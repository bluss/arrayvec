use std::ptr;
use std::slice;

use crate::{CapacityError, LenUint};

/// Implements basic arrayvec methods - based on a few required methods
/// for length and element access.
pub(crate) trait ArrayVecImpl {
    type Item;
    const CAPACITY: usize;

    fn len(&self) -> usize;

    unsafe fn set_len(&mut self, new_len: usize);

    /// Return a slice containing all elements of the vector.
    fn as_slice(&self) -> &[Self::Item] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts(self.as_ptr(), len)
        }
    }

    /// Return a mutable slice containing all elements of the vector.
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        let len = self.len();
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr(), len)
        }
    }

    /// Return a raw pointer to the vector's buffer.
    fn as_ptr(&self) -> *const Self::Item;

    /// Return a raw mutable pointer to the vector's buffer.
    fn as_mut_ptr(&mut self) -> *mut Self::Item;

    fn push(&mut self, element: Self::Item) {
        self.try_push(element).unwrap()
    }

    fn try_push(&mut self, element: Self::Item) -> Result<(), CapacityError<Self::Item>> {
        if self.len() < Self::CAPACITY {
            unsafe {
                self.push_unchecked(element);
            }
            Ok(())
        } else {
            Err(CapacityError::new(element))
        }
    }

    unsafe fn push_unchecked(&mut self, element: Self::Item) {
        let len = self.len();
        debug_assert!(len < Self::CAPACITY);
        ptr::write(self.as_mut_ptr().add(len), element);
        self.set_len(len + 1);
    }

    fn pop(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let new_len = self.len() - 1;
            self.set_len(new_len);
            Some(ptr::read(self.as_ptr().add(new_len)))
        }
    }

    fn clear(&mut self) {
        self.truncate(0)
    }

    fn truncate(&mut self, new_len: usize) {
        unsafe {
            let len = self.len();
            if new_len < len {
                self.set_len(new_len);
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr().add(new_len), len - new_len);
                ptr::drop_in_place(tail);
            }
        }
    }

    fn len_mut(&mut self) -> &mut LenUint;

    /// Extend the ArrayVec from the iterable.
    ///
    /// ## Safety
    ///
    /// Unsafe because if CHECK is false, the length of the input is not checked.
    /// The caller must ensure the length of the input fits in the capacity.
    unsafe fn extend_from_iter<I, const CHECK: bool>(&mut self, iterable: I)
        where I: IntoIterator<Item = Self::Item>
    {
        let take = Self::CAPACITY - self.len();
        let len = self.len();
        let mut ptr = raw_ptr_add(self.as_mut_ptr(), len);
        let end_ptr = raw_ptr_add(ptr, take);
        // Keep the length in a separate variable, write it back on scope
        // exit. To help the compiler with alias analysis and stuff.
        // We update the length to handle panic in the iteration of the
        // user's iterator.
        let mut guard = ScopeExitGuard {
            value: self.len_mut(),
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

    /// Extend the ArrayVec with copies of elements from the slice;
    /// the length of the slice must be <= the remaining capacity in the ArrayVec.
    fn extend_from_slice(&mut self, slice: &[Self::Item])
    where
        Self::Item: Clone
    {
        let take = Self::CAPACITY - self.len();
        debug_assert!(slice.len() <= take);
        unsafe {
            let slice = if take < slice.len() { &slice[..take] } else { slice };
            self.extend_from_iter::<_, false>(slice.iter().cloned());
        }
    }
}

#[inline(never)]
#[cold]
fn extend_panic() {
    panic!("ArrayVec: capacity exceeded in extend/from_iter");
}

struct ScopeExitGuard<T, Data, F>
    where F: FnMut(&Data, &mut T)
{
    pub(crate) value: T,
    pub(crate) data: Data,
    pub(crate) f: F,
}

impl<T, Data, F> Drop for ScopeExitGuard<T, Data, F>
    where F: FnMut(&Data, &mut T)
{
    fn drop(&mut self) {
        (self.f)(&self.data, &mut self.value)
    }
}

/// Rawptr add but uses arithmetic distance for ZST
unsafe fn raw_ptr_add<T>(ptr: *mut T, offset: usize) -> *mut T {
    if std::mem::size_of::<T>() == 0 {
        // Special case for ZST
        (ptr as usize).wrapping_add(offset) as _
    } else {
        ptr.add(offset)
    }
}
