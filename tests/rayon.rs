#![cfg(feature = "rayon")]

use arrayvec::ArrayVec;

// Adapted from `rayon/tests/producer_split_at.rs`

use rayon::iter::plumbing::*;
use rayon::prelude::*;

fn check<F, I>(expected: &[I::Item], mut f: F)
where
    F: FnMut() -> I,
    I: IntoParallelIterator,
    I::Iter: IndexedParallelIterator,
    I::Item: PartialEq + std::fmt::Debug,
{
    map_triples(expected.len() + 1, |i, j, k| {
        Split::forward(f(), i, j, k, expected);
        Split::reverse(f(), i, j, k, expected);
    });
}

fn map_triples<F>(end: usize, mut f: F)
where
    F: FnMut(usize, usize, usize),
{
    for i in 0..end {
        for j in i..end {
            for k in j..end {
                f(i, j, k);
            }
        }
    }
}

#[derive(Debug)]
struct Split {
    i: usize,
    j: usize,
    k: usize,
    reverse: bool,
}

impl Split {
    fn forward<I>(iter: I, i: usize, j: usize, k: usize, expected: &[I::Item])
    where
        I: IntoParallelIterator,
        I::Iter: IndexedParallelIterator,
        I::Item: PartialEq + std::fmt::Debug,
    {
        let result = iter.into_par_iter().with_producer(Split {
            i,
            j,
            k,
            reverse: false,
        });
        assert_eq!(result, expected);
    }

    fn reverse<I>(iter: I, i: usize, j: usize, k: usize, expected: &[I::Item])
    where
        I: IntoParallelIterator,
        I::Iter: IndexedParallelIterator,
        I::Item: PartialEq + std::fmt::Debug,
    {
        let result = iter.into_par_iter().with_producer(Split {
            i,
            j,
            k,
            reverse: true,
        });
        assert!(result.iter().eq(expected.iter().rev()));
    }
}

impl<T> ProducerCallback<T> for Split {
    type Output = Vec<T>;

    fn callback<P>(self, producer: P) -> Self::Output
    where
        P: Producer<Item = T>,
    {
        println!("{:?}", self);

        // Splitting the outer indexes first gets us an arbitrary mid section,
        // which we then split further to get full test coverage.
        let (left, d) = producer.split_at(self.k);
        let (a, mid) = left.split_at(self.i);
        let (b, c) = mid.split_at(self.j - self.i);

        let a = a.into_iter();
        let b = b.into_iter();
        let c = c.into_iter();
        let d = d.into_iter();

        check_len(&a, self.i);
        check_len(&b, self.j - self.i);
        check_len(&c, self.k - self.j);

        let chain = a.chain(b).chain(c).chain(d);
        if self.reverse {
            chain.rev().collect()
        } else {
            chain.collect()
        }
    }
}

fn check_len<I: ExactSizeIterator>(iter: &I, len: usize) {
    assert_eq!(iter.size_hint(), (len, Some(len)));
    assert_eq!(iter.len(), len);
}

// Actual tests

#[test]
fn rayon_arrayvec_producer_split_at() {
    let v: ArrayVec<[u8; 10]> = (0..10).collect();
    check(&v, || v.clone());
}

#[test]
fn rayon_arrayvec_collect() {
    // Iterator length == capacity
    let v: ArrayVec<[u8; 10]> = (0..10u8).into_par_iter().collect();
    assert_eq!(v.len(), 10);

    // Iterator length > capacity
    let v: ArrayVec<[u8; 10]> = (0..20u8).into_par_iter().collect();
    assert_eq!(v.len(), 10);

    // Iterator length < capacity
    let v: ArrayVec<[u8; 10]> = (0..5u8).into_par_iter().collect();
    assert_eq!(v.len(), 5);
}

#[test]
fn rayon_arrayvec_extend() {
    let mut v = ArrayVec::<[u8; 20]>::new();

    // Iterator length == remaining capacity
    v.extend(0..10);
    v.par_extend(0..10u8);
    assert_eq!(v.len(), 20);
    v.clear();

    // Iterator length > remaining capacity
    v.extend(0..10);
    v.par_extend(0..30u8);
    assert_eq!(v.len(), 20);
    v.clear();

    // Iterator length < remaining capacity
    v.extend(0..10);
    v.par_extend(0..5u8);
    assert_eq!(v.len(), 15);
    v.clear();
}
