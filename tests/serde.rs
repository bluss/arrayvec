#![cfg(feature = "serde-1")]
extern crate arrayvec;
extern crate serde_test;

mod array_vec {
    use arrayvec::ArrayVec;

    use serde_test::{Token, assert_tokens};

    #[test]
    fn test_ser_de_empty() {
        let vec = ArrayVec::<[u32; 0]>::new();

        assert_tokens(&vec, &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ]);
    }


    #[test]
    fn test_ser_de() {
        let mut vec = ArrayVec::<[u32; 3]>::new();
        vec.push(20);
        vec.push(55);
        vec.push(123);

        assert_tokens(&vec, &[
            Token::Seq { len: Some(3) },
            Token::U32(20),
            Token::U32(55),
            Token::U32(123),
            Token::SeqEnd,
        ]);
    }
}

mod array_string {
    use arrayvec::ArrayString;

    use serde_test::{Token, assert_tokens};

    #[test]
    fn test_ser_de_empty() {
        let string = ArrayString::<[u8; 0]>::new();

        assert_tokens(&string, &[
            Token::Str(""),
        ]);
    }


    #[test]
    fn test_ser_de() {
        let string = ArrayString::<[u8; 9]>::from("1234 abcd")
            .expect("expected exact specified capacity to be enough");

        assert_tokens(&string, &[
            Token::Str("1234 abcd"),
        ]);
    }
}
