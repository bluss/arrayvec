use std::ops::{
    RangeFull,
    RangeFrom,
    RangeTo,
    Range,
};

/// **RangeArgument** is implemented by Rust's built-in range types, produced
/// by range syntax like `..`, `a..`, `..b` or `c..d`.
pub trait RangeArgument {
    #[doc(hidden)]
    /// Start index (inclusive)
    fn start(&self) -> Option<usize> { None }
    #[doc(hidden)]
    /// End index (exclusive)
    fn end(&self) -> Option<usize> { None }
}


impl RangeArgument for RangeFull {}

impl RangeArgument for RangeFrom<usize> {
    fn start(&self) -> Option<usize> { Some(self.start) }
}

impl RangeArgument for RangeTo<usize> {
    fn end(&self) -> Option<usize> { Some(self.end) }
}

impl RangeArgument for Range<usize> {
    fn start(&self) -> Option<usize> { Some(self.start) }
    fn end(&self) -> Option<usize> { Some(self.end) }
}

