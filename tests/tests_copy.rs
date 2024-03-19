#[allow(unused)]
#[macro_use]
extern crate matches;

#[cfg(feature = "copy")]
mod copy_tests {
    extern crate arrayvec;

    use arrayvec::copy::ArrayVecCopy;
    use arrayvec::CapacityError;
    use std::mem;

    #[test]
    fn test_simple() {
        let mut vec: ArrayVecCopy<usize, 3> = ArrayVecCopy::new();

        vec.push(4);
        vec.push(10);
        vec.push(11);

        let real_vec: Vec<usize> = vec![4, 10, 11];

        for (elt, vec_elt) in vec.iter().zip(real_vec.iter()) {
            assert_eq!(elt, vec_elt);
        }

        assert_eq!(*vec, *real_vec);
    }

    #[test]
    fn test_capacity_left() {
        let mut vec: ArrayVecCopy<usize, 4> = ArrayVecCopy::new();
        assert_eq!(vec.remaining_capacity(), 4);
        vec.push(1);
        assert_eq!(vec.remaining_capacity(), 3);
        vec.push(2);
        assert_eq!(vec.remaining_capacity(), 2);
        vec.push(3);
        assert_eq!(vec.remaining_capacity(), 1);
        vec.push(4);
        assert_eq!(vec.remaining_capacity(), 0);
    }

    #[test]
    fn test_extend_from_slice() {
        let mut vec: ArrayVecCopy<usize, 10> = ArrayVecCopy::new();

        vec.try_extend_from_slice(&[1, 2, 3]).unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(&vec[..], &[1, 2, 3]);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(&vec[..], &[1, 2]);
    }

    #[test]
    fn test_extend_from_slice_error() {
        let mut vec: ArrayVecCopy<usize, 10> = ArrayVecCopy::new();

        vec.try_extend_from_slice(&[1, 2, 3]).unwrap();
        let res = vec.try_extend_from_slice(&[0; 8]);
        assert_matches!(res, Err(_));

        let mut vec: ArrayVecCopy<usize, 0> = ArrayVecCopy::new();
        let res = vec.try_extend_from_slice(&[0; 1]);
        assert_matches!(res, Err(_));
    }

    #[test]
    fn test_try_from_slice_error() {
        use std::convert::TryInto as _;

        let res: Result<ArrayVecCopy<_, 2>, _> = (&[1, 2, 3] as &[_]).try_into();
        assert_matches!(res, Err(_));
    }

    #[test]
    fn test_u16_index() {
        const N: usize = 4096;
        let mut vec: ArrayVecCopy<_, N> = ArrayVecCopy::new();
        for _ in 0..N {
            assert!(vec.try_push(1u8).is_ok());
        }
        assert!(vec.try_push(0).is_err());
        assert_eq!(vec.len(), N);
    }

    #[test]
    fn test_iter() {
        let mut iter = ArrayVecCopy::from([1, 2, 3]).into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_extend() {
        let mut range = 0..10;

        let mut array: ArrayVecCopy<_, 5> = range.by_ref().take(5).collect();
        assert_eq!(&array[..], &[0, 1, 2, 3, 4]);
        assert_eq!(range.next(), Some(5));

        array.extend(range.by_ref().take(0));
        assert_eq!(range.next(), Some(6));

        let mut array: ArrayVecCopy<_, 10> = (0..3).collect();
        assert_eq!(&array[..], &[0, 1, 2]);
        array.extend(3..5);
        assert_eq!(&array[..], &[0, 1, 2, 3, 4]);
    }

    #[should_panic]
    #[test]
    fn test_extend_capacity_panic_1() {
        let mut range = 0..10;

        let _: ArrayVecCopy<_, 5> = range.by_ref().collect();
    }

    #[should_panic]
    #[test]
    fn test_extend_capacity_panic_2() {
        let mut range = 0..10;

        let mut array: ArrayVecCopy<_, 5> = range.by_ref().take(5).collect();
        assert_eq!(&array[..], &[0, 1, 2, 3, 4]);
        assert_eq!(range.next(), Some(5));
        array.extend(range.by_ref().take(1));
    }

    #[test]
    fn test_is_send_sync() {
        let data = ArrayVecCopy::<u8, 5>::new();
        &data as &dyn Send;
        &data as &dyn Sync;
    }

    #[test]
    fn test_compact_size() {
        // 4 bytes + padding + length
        type ByteArray = ArrayVecCopy<u8, 4>;
        println!("{}", mem::size_of::<ByteArray>());
        assert!(mem::size_of::<ByteArray>() <= 2 * mem::size_of::<u32>());

        // just length
        type EmptyArray = ArrayVecCopy<u8, 0>;
        println!("{}", mem::size_of::<EmptyArray>());
        assert!(mem::size_of::<EmptyArray>() <= mem::size_of::<u32>());

        // 3 elements + padding + length
        type QuadArray = ArrayVecCopy<u32, 3>;
        println!("{}", mem::size_of::<QuadArray>());
        assert!(mem::size_of::<QuadArray>() <= 4 * 4 + mem::size_of::<u32>());
    }

    #[test]
    fn test_still_works_with_option_arrayvec() {
        type RefArray = ArrayVecCopy<&'static i32, 2>;
        let array = Some(RefArray::new());
        assert!(array.is_some());
        println!("{:?}", array);
    }

    #[test]
    fn test_drain() {
        let mut v = ArrayVecCopy::from([0; 8]);
        v.pop();
        v.drain(0..7);
        assert_eq!(&v[..], &[]);

        v.extend(0..8);
        v.drain(1..4);
        assert_eq!(&v[..], &[0, 4, 5, 6, 7]);
        let u: ArrayVecCopy<_, 3> = v.drain(1..4).rev().collect();
        assert_eq!(&u[..], &[6, 5, 4]);
        assert_eq!(&v[..], &[0, 7]);
        v.drain(..);
        assert_eq!(&v[..], &[]);
    }

    #[test]
    fn test_drain_range_inclusive() {
        let mut v = ArrayVecCopy::from([0; 8]);
        v.drain(0..=7);
        assert_eq!(&v[..], &[]);

        v.extend(0..8);
        v.drain(1..=4);
        assert_eq!(&v[..], &[0, 5, 6, 7]);
        let u: ArrayVecCopy<_, 3> = v.drain(1..=2).rev().collect();
        assert_eq!(&u[..], &[6, 5]);
        assert_eq!(&v[..], &[0, 7]);
        v.drain(..);
        assert_eq!(&v[..], &[]);
    }

    #[test]
    #[should_panic]
    fn test_drain_range_inclusive_oob() {
        let mut v = ArrayVecCopy::from([0; 0]);
        v.drain(0..=0);
    }

    #[test]
    fn test_retain() {
        let mut v = ArrayVecCopy::from([0; 8]);
        for (i, elt) in v.iter_mut().enumerate() {
            *elt = i;
        }
        v.retain(|_| true);
        assert_eq!(&v[..], &[0, 1, 2, 3, 4, 5, 6, 7]);
        v.retain(|elt| {
            *elt /= 2;
            *elt % 2 == 0
        });
        assert_eq!(&v[..], &[0, 0, 2, 2]);
        v.retain(|_| false);
        assert_eq!(&v[..], &[]);
    }

    #[test]
    #[should_panic]
    fn test_drain_oob() {
        let mut v = ArrayVecCopy::from([0; 8]);
        v.pop();
        v.drain(0..8);
    }

    #[test]
    fn test_insert() {
        let mut v = ArrayVecCopy::from([]);
        assert_matches!(v.try_push(1), Err(_));

        let mut v = ArrayVecCopy::<_, 3>::new();
        v.insert(0, 0);
        v.insert(1, 1);
        //let ret1 = v.try_insert(3, 3);
        //assert_matches!(ret1, Err(InsertError::OutOfBounds(_)));
        assert_eq!(&v[..], &[0, 1]);
        v.insert(2, 2);
        assert_eq!(&v[..], &[0, 1, 2]);

        let ret2 = v.try_insert(1, 9);
        assert_eq!(&v[..], &[0, 1, 2]);
        assert_matches!(ret2, Err(_));

        let mut v = ArrayVecCopy::from([2]);
        assert_matches!(v.try_insert(0, 1), Err(CapacityError { .. }));
        assert_matches!(v.try_insert(1, 1), Err(CapacityError { .. }));
        //assert_matches!(v.try_insert(2, 1), Err(CapacityError { .. }));
    }

    #[test]
    fn test_into_inner_1() {
        let mut v = ArrayVecCopy::from([1, 2]);
        v.pop();
        let u = v.clone();
        assert_eq!(v.into_inner(), Err(u));
    }

    #[test]
    fn test_into_inner_2() {
        let mut v = ArrayVecCopy::<char, 4>::new();
        v.push('a');
        v.push('b');
        v.push('c');
        v.push('d');
        assert_eq!(v.into_inner().unwrap(), ['a', 'b', 'c', 'd']);
    }

    #[test]
    fn test_into_inner_3() {
        let mut v = ArrayVecCopy::<i32, 4>::new();
        v.extend(1..=4);
        assert_eq!(v.into_inner().unwrap(), [1, 2, 3, 4]);
    }

    #[test]
    fn test_take() {
        let mut v1 = ArrayVecCopy::<i32, 4>::new();
        v1.extend(1..=4);
        let v2 = v1.take();
        assert!(v1.into_inner().is_err());
        assert_eq!(v2.into_inner().unwrap(), [1, 2, 3, 4]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_write() {
        use std::io::Write;
        let mut v = ArrayVecCopy::<_, 8>::new();
        write!(&mut v, "\x01\x02\x03").unwrap();
        assert_eq!(&v[..], &[1, 2, 3]);
        let r = v.write(&[9; 16]).unwrap();
        assert_eq!(r, 5);
        assert_eq!(&v[..], &[1, 2, 3, 9, 9, 9, 9, 9]);
    }

    #[test]
    fn array_clone_from() {
        let mut v = ArrayVecCopy::<usize, 4>::new();
        v.push(1);
        v.push(5);
        v.push(6);
        let reference = v.to_vec();
        let mut u = ArrayVecCopy::<usize, 4>::new();
        u.clone_from(&v);
        assert_eq!(&u, &reference[..]);

        let mut t = ArrayVecCopy::<usize, 4>::new();
        t.push(97);
        t.push(0);
        t.push(5);
        t.push(2);
        t.clone_from(&v);
        assert_eq!(&t, &reference[..]);
        t.clear();
        t.clone_from(&v);
        assert_eq!(&t, &reference[..]);
    }

    #[test]
    fn test_insert_at_length() {
        let mut v = ArrayVecCopy::<_, 8>::new();
        let result1 = v.try_insert(0, "a");
        let result2 = v.try_insert(1, "b");
        assert!(result1.is_ok() && result2.is_ok());
        assert_eq!(&v[..], &["a", "b"]);
    }

    #[should_panic]
    #[test]
    fn test_insert_out_of_bounds() {
        let mut v = ArrayVecCopy::<_, 8>::new();
        let _ = v.try_insert(1, "test");
    }

    #[test]
    fn test_pop_at() {
        let mut v = ArrayVecCopy::<&'static str, 4>::new();
        v.push("a");
        v.push("b");
        v.push("c");
        v.push("d");

        assert_eq!(v.pop_at(4), None);
        assert_eq!(v.pop_at(1), Some("b"));
        assert_eq!(v.pop_at(1), Some("c"));
        assert_eq!(v.pop_at(2), None);
        assert_eq!(&v[..], &["a", "d"]);
    }

    #[test]
    fn test_sizes() {
        let v = ArrayVecCopy::from([0u8; 1 << 16]);
        assert_eq!(vec![0u8; v.len()], &v[..]);
    }

    #[cfg(feature = "array-sizes-33-128")]
    #[test]
    fn test_sizes_33_128() {
        ArrayVecCopy::from([0u8; 52]);
        ArrayVecCopy::from([0u8; 127]);
    }

    #[cfg(feature = "array-sizes-129-255")]
    #[test]
    fn test_sizes_129_255() {
        ArrayVecCopy::from([0u8; 237]);
        ArrayVecCopy::from([0u8; 255]);
    }

    #[test]
    fn test_extend_zst() {
        let mut range = 0..10;
        #[derive(Copy, Clone, PartialEq, Debug)]
        struct Z; // Zero sized type

        let mut array: ArrayVecCopy<_, 5> = range.by_ref().take(5).map(|_| Z).collect();
        assert_eq!(&array[..], &[Z; 5]);
        assert_eq!(range.next(), Some(5));

        array.extend(range.by_ref().take(0).map(|_| Z));
        assert_eq!(range.next(), Some(6));

        let mut array: ArrayVecCopy<_, 10> = (0..3).map(|_| Z).collect();
        assert_eq!(&array[..], &[Z; 3]);
        array.extend((3..5).map(|_| Z));
        assert_eq!(&array[..], &[Z; 5]);
        assert_eq!(array.len(), 5);
    }

    #[test]
    fn allow_max_capacity_arrayvec_type() {
        // this type is allowed to be used (but can't be constructed)
        let _v: ArrayVecCopy<(), { usize::MAX }>;
    }

    #[should_panic(expected = "largest supported capacity")]
    #[test]
    fn deny_max_capacity_arrayvec_value() {
        if mem::size_of::<usize>() <= mem::size_of::<u32>() {
            panic!("This test does not work on this platform. 'largest supported capacity'");
        }
        // this type is allowed to be used (but can't be constructed)
        let _v: ArrayVecCopy<(), { usize::MAX }> = ArrayVecCopy::new();
    }

    #[should_panic(expected = "index out of bounds")]
    #[test]
    fn deny_max_capacity_arrayvec_value_const() {
        if mem::size_of::<usize>() <= mem::size_of::<u32>() {
            panic!("This test does not work on this platform. 'index out of bounds'");
        }
        // this type is allowed to be used (but can't be constructed)
        let _v: ArrayVecCopy<(), { usize::MAX }> = ArrayVecCopy::new_const();
    }

    #[test]
    fn test_arrayvec_const_constructible() {
        const OF_U8: ArrayVecCopy<u8, 10> = ArrayVecCopy::new_const();

        let mut var = OF_U8;
        assert!(var.is_empty());
        assert_eq!(var, ArrayVecCopy::new());
        var.push(3);
        assert_eq!(var[..], [3]);
    }
}
