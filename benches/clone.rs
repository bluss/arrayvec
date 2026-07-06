
extern crate arrayvec;
#[macro_use] extern crate bencher;

use arrayvec::ArrayVec;

use bencher::Bencher;
use bencher::black_box;

fn clone_x<const N: usize>(b: &mut Bencher) {
    let mut v = ArrayVec::<u8, N>::new();
    v.extend((0..N).map(|x| x as u8));
    b.iter(|| {
        black_box(v.clone())
    });
    b.bytes = v.len() as u64;
}

fn clone_8(b: &mut Bencher) {
    clone_x::<8>(b);
}
fn clone_16(b: &mut Bencher) {
    clone_x::<16>(b);
}
fn clone_32(b: &mut Bencher) {
    clone_x::<32>(b);
}
fn clone_64(b: &mut Bencher) {
    clone_x::<64>(b);
}
fn clone_512(b: &mut Bencher) {
    clone_x::<512>(b);
}

benchmark_group!(benches,
                 clone_8,
                 clone_16,
                 clone_32,
                 clone_64,
                 clone_512
);

benchmark_main!(benches);
