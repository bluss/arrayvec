use arrayvec::{ArrayString, ArrayVec};
fn main() {
    let mut v: ArrayVec<u32, 8> = ArrayVec::new();
    v.extend([10, 20, 30]);
    let mut names: ArrayVec<String, 4> = ArrayVec::new();
    names.push("alpha".into());
    names.push("beta".into());
    let s: ArrayString<16> = ArrayString::from("hello").unwrap();
    std::hint::black_box((&v, &names, &s)); // break here
}