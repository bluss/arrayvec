
use std::ops::{
    RangeFull,
    RangeFrom,
    RangeTo,
    Range,
};

/// For illustration, add an inclusive range too
#[derive(Copy, Clone)]
pub struct InclusiveRange<T> {
    pub start: T,
    pub end: T,
}

pub trait IntoIndexRange {
    fn into_index_range(self, len: usize) -> Range<usize>;
}

pub unsafe trait IntoCheckedRange {
    fn into_checked_range(self, len: usize) -> Option<Range<usize>>;
}


impl IntoIndexRange for RangeFull {
    #[inline]
    fn into_index_range(self, len: usize) -> Range<usize> {
        0..len
    }
}

impl IntoIndexRange for RangeFrom<usize> {
    #[inline]
    fn into_index_range(self, len: usize) -> Range<usize> {
        self.start..len
    }
}

impl IntoIndexRange for RangeTo<usize> {
    #[inline]
    fn into_index_range(self, _len: usize) -> Range<usize> {
        0..self.end
    }
}

impl IntoIndexRange for Range<usize> {
    #[inline]
    fn into_index_range(self, _len: usize) -> Range<usize> {
        self
    }
}

impl IntoIndexRange for InclusiveRange<usize> {
    #[inline]
    fn into_index_range(self, _len: usize) -> Range<usize> {
        // this doesn't work so well!
        self.start..self.end.saturating_add(1)
    }
}


unsafe impl IntoCheckedRange for RangeFull {
    #[inline]
    fn into_checked_range(self, len: usize) -> Option<Range<usize>> {
        Some(0..len)
    }
}

unsafe impl IntoCheckedRange for RangeFrom<usize> {
    #[inline]
    fn into_checked_range(self, len: usize) -> Option<Range<usize>> {
        if self.start <= len {
            Some(self.into_index_range(len))
        } else { None }
    }
}

unsafe impl IntoCheckedRange for RangeTo<usize> {
    #[inline]
    fn into_checked_range(self, len: usize) -> Option<Range<usize>> {
        if self.end <= len {
            Some(self.into_index_range(len))
        } else { None }
    }
}

unsafe impl IntoCheckedRange for Range<usize> {
    #[inline]
    fn into_checked_range(self, len: usize) -> Option<Range<usize>> {
        if self.start <= self.end && self.end <= len {
            Some(self.into_index_range(len))
        } else { None }
    }
}

unsafe impl IntoCheckedRange for InclusiveRange<usize> {
    #[inline]
    // this doesn't work so well
    fn into_checked_range(self, len: usize) -> Option<Range<usize>> {
        if self.start <= self.end && self.end < len {
            Some(self.start..self.end + 1)
        } else { None }
    }
}
