
extern crate arrayvec;
#[macro_use] extern crate bencher;

use arrayvec::ArrayVec;

use bencher::Bencher;
use bencher::black_box;

fn sum_x<const N: usize>(b: &mut Bencher) {
    let mut v = ArrayVec::<u8, N>::new();
    v.extend((0..N).map(|x| x as u8));
    b.iter(|| {
        let mut overall_sum: u8 = 0;
        for _ in 0..100 {
            let v = black_box(&v);
            let mut sum: u8 = 0;
            for i in 0..v.len() {
                sum = sum.wrapping_add(v[i]);
            }
            overall_sum = overall_sum.wrapping_add(black_box(sum));
        }
        black_box(overall_sum)
    });
    b.bytes = v.len() as u64;
}

fn sum_1(b: &mut Bencher) {
    sum_x::<1>(b);
}
fn sum_16(b: &mut Bencher) {
    sum_x::<16>(b);
}
fn sum_128(b: &mut Bencher) {
    sum_x::<128>(b);
}

benchmark_group!(benches,
                 sum_1,
                 sum_16,
                 sum_128,
);

benchmark_main!(benches);
