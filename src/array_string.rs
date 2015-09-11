use std::borrow::Borrow;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::str;

use array::Array;
use array_vec::ArrayVec;


/// A string with a fixed capacity.
///
/// The `ArrayString` is a string backed by a fixed size array. It keeps track
/// of its length.
///
/// The string is a contiguous value that you can store directly on the stack
/// if needed.
///
/// Due to technical restrictions, this struct does not implement `Copy` even
/// though it would be safe to do so.
pub struct ArrayString<A: Array<Item=u8>> {
    vec: ArrayVec<A>,
}

impl<A: Array<Item=u8>> ArrayString<A> {
    /// Create a new empty `ArrayString`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 16]>::new();
    /// string.push_str("foo");
    /// assert_eq!(&string[..], "foo");
    /// assert_eq!(string.capacity(), 16);
    /// ```
    pub fn new() -> ArrayString<A> {
        ArrayString {
            vec: ArrayVec::new()
        }
    }

    /// Return the capacity of the `ArrayString`.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let string = ArrayString::<[_; 3]>::new();
    /// assert_eq!(string.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { self.vec.capacity() }

    /// Adds the given character to the end of the string.
    ///
    /// Returns `None` if the push succeeds, or and returns `Some(c)` if the
    /// backing array is not large enough to fit the additional character.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// let none1 = string.push('a');
    /// let none2 = string.push('b');
    /// let overflow = string.push('c');
    ///
    /// assert_eq!(&string[..], "ab");
    /// assert_eq!(none1, None);
    /// assert_eq!(none2, None);
    /// assert_eq!(overflow, Some('c'));
    /// ```
    pub fn push(&mut self, c: char) -> Option<char> {
        use std::fmt::Write;
        self.write_char(c).err().map(|_| c)
    }

    /// Adds the given string slice to the end of the string.
    ///
    /// Returns `None` if the push succeeds, or and returns `Some(s)` if the
    /// backing array is not large enough to fit the string.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// let none1 = string.push_str("a");
    /// let overflow1 = string.push_str("bc");
    /// let none2 = string.push_str("d");
    /// let overflow2 = string.push_str("ef");
    ///
    /// assert_eq!(&string[..], "ad");
    /// assert_eq!(none1, None);
    /// assert_eq!(none2, None);
    /// assert_eq!(overflow1, Some("bc"));
    /// assert_eq!(overflow2, Some("ef"));
    /// ```
    pub fn push_str<'a>(&mut self, s: &'a str) -> Option<&'a str> {
        if self.len() + s.len() > self.capacity() {
            return Some(s);
        }
        let mut bytes = s.bytes();
        self.vec.extend(&mut bytes);
        assert!(bytes.next().is_none());
        None
    }



    /// Make the string empty.
    pub fn clear(&mut self) {
        mem::replace(self, ArrayString::new());
    }

    /// Set the strings's length.
    ///
    /// May panic if `length` is greater than the capacity.
    ///
    /// This function is `unsafe` because it changes the notion of the
    /// number of “valid” characters in the string. Use with care.
    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.vec.set_len(length)
    }
}

impl<A: Array<Item=u8>> Deref for ArrayString<A> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.vec) }
    }
}

impl<A: Array<Item=u8>> Borrow<str> for ArrayString<A> {
    fn borrow(&self) -> &str { self }
}

impl<A: Array<Item=u8>> AsRef<str> for ArrayString<A> {
    fn as_ref(&self) -> &str { self }
}

impl<A: Array<Item=u8>> fmt::Debug for ArrayString<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

/// `Write` appends written data to the end of the string.
impl<A: Array<Item=u8>> fmt::Write for ArrayString<A> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.push_str(s) {
            None => Ok(()),
            Some(_) => Err(fmt::Error),
        }
    }
}

//#[derive(Clone, /*Copy,*/ Eq, Hash, Ord, PartialEq, PartialOrd)]
impl<A: Array<Item=u8>> Clone for ArrayString<A> {
    fn clone(&self) -> ArrayString<A> {
        ArrayString { vec: self.vec.clone() }
    }
}

