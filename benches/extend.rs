extern crate arrayvec;
#[macro_use] extern crate bencher;

use std::io::Write;
use std::hint::black_box;

const CAP: usize = 1024 * 5;

use arrayvec::ArrayVec;
use bencher::Bencher;

fn extend_with_constant(b: &mut Bencher) {
    let mut v = ArrayVec::<u8, CAP>::new();
    b.iter(|| {
        v.clear();
        let constant = black_box(255u8);
        v.extend((0..CAP).map(move |_| constant));
        black_box(v.as_ptr());
        black_box(v[CAP - 1])
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_range(b: &mut Bencher) {
    let mut v = ArrayVec::<u8, CAP>::new();
    b.iter(|| {
        v.clear();
        let r = black_box((0..CAP).map(|x| x as u8));
        v.extend(r);
        black_box(v.as_ptr());
        black_box(v[CAP - 1])
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_slice(b: &mut Bencher) {
    let data = [1u8; CAP];
    let mut v = ArrayVec::<u8, CAP>::new();
    b.iter(|| {
        v.clear();
        let data = black_box(&data[..]);
        v.extend(data.iter().copied());
        black_box(v.as_ptr());
        black_box(v[CAP - 1])
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_write(b: &mut Bencher) {
    let data = [1u8; CAP];
    let mut v = ArrayVec::<u8, CAP>::new();
    b.iter(|| {
        v.clear();
        let data = black_box(&data[..]);
        v.write(data).ok();
        black_box(v[CAP - 1])
    });
    b.bytes = v.capacity() as u64;
}

fn extend_from_slice(b: &mut Bencher) {
    let data = [1u8; CAP];
    let mut v = ArrayVec::<u8, CAP>::new();
    b.iter(|| {
        v.clear();
        let data = black_box(&data[..]);
        v.try_extend_from_slice(data).ok();
        black_box(v.as_ptr());
        black_box(v[CAP - 1]);
    });
    b.bytes = v.capacity() as u64;
}

benchmark_group!(benches,
                 extend_with_constant,
                 extend_with_range,
                 extend_with_slice,
                 extend_with_write,
                 extend_from_slice
);

benchmark_main!(benches);
