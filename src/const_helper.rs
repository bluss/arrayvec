use core::mem::MaybeUninit;

use crate::ArrayString;

/// Creates a const ArrayString from a str slice.
pub const fn str<const CAP: usize>(s: &str) -> ArrayString<CAP> {
    assert_capacity_limit_const!(CAP);

    let bytes = s.as_bytes();
    let len = bytes.len();

    // Check that capacity is not exceeded
    if len > CAP {
        panic!("ArrayString: capacity exceeded in const_str");
    }

    let mut xs = [MaybeUninit::<u8>::uninit(); CAP];
    let mut i = 0;
    while i < len {
        xs[i] = MaybeUninit::new(bytes[i]);
        i += 1;
    }

    // Safety: We have initialized `len` bytes in `xs`
    // and ensured that `len <= CAP` before
    // and s is a valid UTF-8 string.
    unsafe { ArrayString::from_raw_parts(xs, len) }
}

/// Create a const ArrayString from a byte slice.
pub const fn byte_str<const CAP: usize>(bytes: &[u8]) -> ArrayString<CAP> {
    // for the const_helper feature MSRV 1.63.0 is required
    #[allow(clippy::incompatible_msrv)]
    let Ok(str) = core::str::from_utf8(bytes) else {
        panic!("ArrayString: invalid UTF-8 in const_byte_str");
    };
    crate::const_helper::str(str)
}

/// Creates a const ArrayString from a str slice.
///
/// # Examples
/// ```rust
/// use arrayvec::array_str;
/// // With inferred capacity
/// const S: arrayvec::ArrayString<5> = array_str!("hello");
/// assert_eq!(&S, "hello");
/// // With specified capacity
/// const S2: arrayvec::ArrayString<10> = array_str!("hello", 10);
/// assert_eq!(&S2, "hello");   
/// assert_eq!(S2.capacity(), 10);
/// ```
#[macro_export]
macro_rules! array_str {
    ($str:expr) => {
        $crate::const_helper::str::<{ $str.len() }>($str)
    };
    ($str:expr, $cap:expr) => {
        $crate::const_helper::str::<$cap>($str)
    };
}

/// Creates a const ArrayString from a byte slice.
///
/// # Examples
/// ```rust
/// use arrayvec::array_bstr;
/// // With inferred capacity
/// const B: arrayvec::ArrayString<5> = array_bstr!(b"hello");
/// assert_eq!(&B, "hello");
/// // With specified capacity
/// const B2: arrayvec::ArrayString<10> = array_bstr!(b"hello", 10);
/// assert_eq!(&B2, "hello");   
/// assert_eq!(B2.capacity(), 10);
/// ```
#[macro_export]
macro_rules! array_bstr {
    ($bstr:expr) => {
        $crate::const_helper::byte_str::<{ $bstr.len() }>($bstr)
    };
    ($bstr:expr, $cap:expr) => {
        $crate::const_helper::byte_str::<$cap>($bstr)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_str() {
        const S_EMPTY: ArrayString<0> = array_str!("");
        assert_eq!(&S_EMPTY, "");

        const S_EMPTY_CAP5: ArrayString<5> = array_str!("", 5);
        assert_eq!(&S_EMPTY_CAP5, "");

        const S1: ArrayString<5> = array_str!("hello");
        assert_eq!(&S1, "hello");

        const S2: ArrayString<10> = array_str!("hello", 10);
        assert_eq!(&S2, "hello");
    }

    #[test]
    fn array_bstr() {
        const B_EMPTY: ArrayString<0> = array_bstr!(b"");
        assert_eq!(&B_EMPTY, "");

        const B_EMPTY_CAP5: ArrayString<5> = array_bstr!(b"", 5);
        assert_eq!(&B_EMPTY_CAP5, "");

        const B1: ArrayString<5> = array_bstr!(b"hello");
        assert_eq!(&B1, "hello");

        const B2: ArrayString<10> = array_bstr!(b"hello", 10);
        assert_eq!(&B2, "hello");
    }

    #[test]
    #[should_panic]
    fn fail_empty() {
        let _bad_empty = array_str!("hello", 0);
    }

    #[test]
    #[should_panic]
    fn fail_bempty() {
        let _bad_bempty = array_bstr!(b"hello", 0);
    }

    #[test]
    #[should_panic]
    fn fail_cap() {
        let _bad_small = array_str!("hello", 4);
    }

    #[test]
    #[should_panic]
    fn fail_bcap() {
        let _bad_bsmall = array_bstr!(b"hello", 4);
    }

    #[test]
    #[should_panic]
    fn fail_utf8() {
        let _bad_utf8 = array_bstr!(b"\xFF\xFF\xFF", 4);
    }
}
