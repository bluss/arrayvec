use core::ptr;
use core::slice;

use crate::Drain;

/// A splicing iterator adapted for `ArrayVec` from `Vec`.
pub struct Splice<
    'a,
    I: Iterator + 'a,
    const CAP: usize
> {
    pub(super) drain: Drain<'a, I::Item, CAP>,
    pub(super) replace_with: I,
}

impl<I: Iterator, const CAP: usize> Iterator for Splice<'_, I, CAP> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<I: Iterator, const CAP: usize> DoubleEndedIterator for Splice<'_, I, CAP> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<I: Iterator, const CAP: usize> ExactSizeIterator for Splice<'_, I, CAP> {}

impl<I: Iterator, const CAP: usize> Drop for Splice<'_, I, CAP> {
    fn drop(&mut self) {
        self.drain.by_ref().for_each(drop);

        unsafe {
            if self.drain.tail_len == 0 {
                let target_vec = &mut *self.drain.vec;
                target_vec.extend(self.replace_with.by_ref());
                return;
            }

            // First fill the range left by drain().
            if !self.drain.fill(&mut self.replace_with) {
                return;
            }

            // There may be more elements. Use the lower bound as an estimate.
            // FIXME: Is the upper bound a better guess? Or something else?
            let (lower_bound, _upper_bound) = self.replace_with.size_hint();
            if lower_bound > 0 {
                if !self.drain.fill(&mut self.replace_with) {
                    return;
                }
            }

            // Collect any remaining elements.
            // This is a zero-length vector which does not allocate if `lower_bound` was exact.
            let mut collected = self.replace_with.by_ref().collect::<Vec<I::Item>>().into_iter();
            // Now we have an exact count.
            if collected.len() > 0 {
                let filled = self.drain.fill(&mut collected);
                debug_assert!(filled);
                debug_assert_eq!(collected.len(), 0);
            }
        }
        // Let `Drain::drop` move the tail back if necessary and restore `vec.len`.
    }
}

/// Private helper methods for `Splice::drop`
impl<T, const CAP: usize> Drain<'_, T, CAP> {
    /// The range from `self.vec.len` to `self.tail_start` contains elements
    /// that have been moved out.
    /// Fill that range as much as possible with new elements from the `replace_with` iterator.
    /// Returns `true` if we filled the entire range. (`replace_with.next()` didn’t return `None`.)
    unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
        let vec = &mut *self.vec;
        let range_start = vec.len as usize;
        let range_end = self.tail_start;
        let range_slice = slice::from_raw_parts_mut(vec.get_unchecked_ptr(range_start), range_end - range_start);

        for place in range_slice {
            if let Some(new_item) = replace_with.next() {
                ptr::write(place, new_item);
                vec.len += 1;
            } else {
                return false;
            }
        }
        true
    }
}
