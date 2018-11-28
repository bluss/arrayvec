
extern crate arrayvec;
#[macro_use] extern crate bencher;

use arrayvec::ArrayVec;

use bencher::Bencher;
use bencher::black_box;

fn extend_with_constant(b: &mut Bencher) {
    let mut v = ArrayVec::<[u8; 512]>::new();
    let cap = v.capacity();
    b.iter(|| {
        v.clear();
        v.extend((0..cap).map(|_| 1));
        v[0]
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_range(b: &mut Bencher) {
    let mut v = ArrayVec::<[u8; 512]>::new();
    let cap = v.capacity();
    b.iter(|| {
        v.clear();
        v.extend((0..cap).map(|x| x as _));
        v[0]
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_slice(b: &mut Bencher) {
    let mut v = ArrayVec::<[u8; 512]>::new();
    let data = [1; 512];
    b.iter(|| {
        v.clear();
        v.extend(black_box(data.iter()).cloned());
        v[0]
    });
    b.bytes = v.capacity() as u64;
}

fn extend_with_slice_fn(b: &mut Bencher) {
    let mut v = ArrayVec::<[u8; 512]>::new();
    let data = [1; 512];
    b.iter(|| {
        v.clear();
        black_box(v.try_extend_from_slice(&data));
        v[0]
    });
    b.bytes = v.capacity() as u64;
}

benchmark_group!(benches, extend_with_constant, extend_with_range, extend_with_slice, extend_with_slice_fn);
benchmark_main!(benches);
