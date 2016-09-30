use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ptr;
use std::ops::{Deref, DerefMut};
use std::str;

use Array;
use CapacityError;
use raw::RawArrayVec;

/// A string with a fixed capacity.
///
/// The `ArrayString` is a string backed by a fixed size array. It keeps track
/// of its length.
///
/// The string is a contiguous value that you can store directly on the stack
/// if needed.
pub struct ArrayString<A: Array<Item=u8>> {
    inner: RawArrayVec<A>,
}

impl<A: Array<Item=u8> + Copy> Copy for ArrayString<A> where A::Item: Clone { }

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
            inner: RawArrayVec::new(),
        }
    }

    /// Create a new `ArrayString` from a `str`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// **Errors** if the backing array is not large enough to fit the string.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 3]>::from("foo").unwrap();
    /// assert_eq!(&string[..], "foo");
    /// assert_eq!(string.len(), 3);
    /// assert_eq!(string.capacity(), 3);
    /// ```
    pub fn from(s: &str) -> Result<Self, CapacityError<&str>> {
        let mut arraystr = Self::new();
        try!(arraystr.push_str(s));
        Ok(arraystr)
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
    pub fn capacity(&self) -> usize { self.inner.capacity() }

    /// Return if the `ArrayString` is completely filled.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 1]>::new();
    /// assert!(!string.is_full());
    /// string.push_str("A");
    /// assert!(string.is_full());
    /// ```
    pub fn is_full(&self) -> bool { self.len() == self.capacity() }

    /// Adds the given char to the end of the string.
    ///
    /// Returns `Ok` if the push succeeds.
    ///
    /// **Errors** if the backing array is not large enough to fit the additional char.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// string.push('a').unwrap();
    /// string.push('b').unwrap();
    /// let overflow = string.push('c');
    ///
    /// assert_eq!(&string[..], "ab");
    /// assert_eq!(overflow.unwrap_err().element(), 'c');
    /// ```
    pub fn push(&mut self, c: char) -> Result<(), CapacityError<char>> {
        use std::fmt::Write;
        self.write_char(c).map_err(|_| CapacityError::new(c))
    }

    /// Adds the given string slice to the end of the string.
    ///
    /// Returns `Ok` if the push succeeds.
    ///
    /// **Errors** if the backing array is not large enough to fit the string.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// string.push_str("a").unwrap();
    /// let overflow1 = string.push_str("bc");
    /// string.push_str("d").unwrap();
    /// let overflow2 = string.push_str("ef");
    ///
    /// assert_eq!(&string[..], "ad");
    /// assert_eq!(overflow1.unwrap_err().element(), "bc");
    /// assert_eq!(overflow2.unwrap_err().element(), "ef");
    /// ```
    pub fn push_str<'a>(&mut self, s: &'a str) -> Result<(), CapacityError<&'a str>> {
        if s.len() > self.capacity() - self.len() {
            return Err(CapacityError::new(s));
        }
        unsafe {
            let dst = self.inner.as_mut_ptr().offset(self.len() as isize);
            let src = s.as_ptr();
            ptr::copy_nonoverlapping(src, dst, s.len());
            let newl = self.len() + s.len();
            self.set_len(newl);
        }
        Ok(())
    }

    /// Make the string empty.
    pub fn clear(&mut self) {
        unsafe {
            self.set_len(0);
        }
    }

    /// Set the strings's length.
    ///
    /// May panic if `length` is greater than the capacity.
    ///
    /// This function is `unsafe` because it changes the notion of the
    /// number of “valid” bytes in the string. Use with care.
    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.inner.set_len(length)
    }

    /// Return a string slice of the whole `ArrayString`.
    pub fn as_str(&self) -> &str {
        self
    }
}

impl<A: Array<Item=u8>> Deref for ArrayString<A> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(&self.inner)
        }
    }
}

impl<A: Array<Item=u8>> DerefMut for ArrayString<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        unsafe {
            let bytes: &mut [u8] = &mut self.inner;
            mem::transmute(bytes)
        }
    }
}

impl<A: Array<Item=u8>> PartialEq for ArrayString<A> {
    fn eq(&self, rhs: &Self) -> bool {
        **self == **rhs
    }
}

impl<A: Array<Item=u8>> PartialEq<str> for ArrayString<A> {
    fn eq(&self, rhs: &str) -> bool {
        &**self == rhs
    }
}

impl<A: Array<Item=u8>> PartialEq<ArrayString<A>> for str {
    fn eq(&self, rhs: &ArrayString<A>) -> bool {
        self == &**rhs
    }
}

impl<A: Array<Item=u8>> Eq for ArrayString<A> { }

impl<A: Array<Item=u8>> Hash for ArrayString<A> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (**self).hash(h)
    }
}

impl<A: Array<Item=u8>> Borrow<str> for ArrayString<A> {
    fn borrow(&self) -> &str { self }
}

impl<A: Array<Item=u8>> AsRef<str> for ArrayString<A> {
    fn as_ref(&self) -> &str { self }
}

impl<A: Array<Item=u8>> fmt::Debug for ArrayString<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

impl<A: Array<Item=u8>> fmt::Display for ArrayString<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

/// `Write` appends written data to the end of the string.
impl<A: Array<Item=u8>> fmt::Write for ArrayString<A> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

impl<A: Array<Item=u8> + Copy> Clone for ArrayString<A> {
    fn clone(&self) -> ArrayString<A> {
        *self
    }
    fn clone_from(&mut self, rhs: &Self) {
        // guaranteed to fit due to types matching.
        self.clear();
        let _ = self.push_str(rhs);
    }
}
