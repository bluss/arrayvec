#![cfg(feature = "serde")]
extern crate arrayvec;
extern crate serde_test;

mod array_vec {
    use arrayvec::ArrayVec;

    use serde_test::{assert_de_tokens_error, assert_tokens, Token};
    use serde_with::{serde_as, DisplayFromStr};

    #[test]
    fn test_ser_de_empty() {
        let vec = ArrayVec::<u32, 0>::new();

        assert_tokens(&vec, &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ]);
    }


    #[test]
    fn test_ser_de() {
        let mut vec = ArrayVec::<u32, 3>::new();
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

    #[test]
    fn test_de_too_large() {
        assert_de_tokens_error::<ArrayVec<u32, 2>>(
            &[
                Token::Seq { len: Some(3) },
                Token::U32(13),
                Token::U32(42),
                Token::U32(68),
            ],
            "invalid length 3, expected an array with no more than 2 items",
        );
    }

    #[serde_as]
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
    struct SerdeAs {
        #[serde_as(as = "ArrayVec<DisplayFromStr, 3>")]
        x: ArrayVec<u32, 3>
    }

    #[test]
    fn test_ser_de_as() {
        let mut serde_as = SerdeAs {x: ArrayVec::<u32, 3>::new()};
        serde_as.x.push(20);
        serde_as.x.push(55);
        serde_as.x.push(123);

        assert_tokens(
            &serde_as,
            &[
                Token::Struct { name: "SerdeAs", len: 1 },
                Token::Str("x"),
                Token::Seq { len: Some(3) },
                Token::Str("20"),
                Token::Str("55"),
                Token::Str("123"),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}

mod array_string {
    use arrayvec::ArrayString;

    use serde_test::{Token, assert_tokens, assert_de_tokens_error};

    #[test]
    fn test_ser_de_empty() {
        let string = ArrayString::<0>::new();

        assert_tokens(&string, &[
            Token::Str(""),
        ]);
    }


    #[test]
    fn test_ser_de() {
        let string = ArrayString::<9>::from("1234 abcd")
            .expect("expected exact specified capacity to be enough");

        assert_tokens(&string, &[
            Token::Str("1234 abcd"),
        ]);
    }

    #[test]
    fn test_de_too_large() {
        assert_de_tokens_error::<ArrayString<2>>(&[
            Token::Str("afd")
        ], "invalid length 3, expected a string no more than 2 bytes long");
    }
}
