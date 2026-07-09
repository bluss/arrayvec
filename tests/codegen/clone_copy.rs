extern crate arrayvec;
use arrayvec::ArrayVec;

#[unsafe(no_mangle)]
#[inline(never)]
pub fn test_subject(array: &ArrayVec<u8, 8>) -> ArrayVec<u8, 8> {
    array.clone()
}
