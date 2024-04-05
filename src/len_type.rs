use core::ops::{Add, Sub};

macro_rules! impl_lenuint {
    ($Sealed:ty, $LenUint:ty, $ty: path) => {
        impl $Sealed for $ty {}
        impl $LenUint for $ty {
            const MAX: usize = <$ty>::MAX as usize;
            const ZERO: Self = 0;
            fn from_usize(n: usize) -> Self { n as $ty }
            fn to_usize(self) -> usize { self as usize }
        }
    };
}

macro_rules! impl_default_lentype_from_cap {
    ($LenT:ty => $($CAP:literal),*) => {
        $(
            impl CapToDefaultLenType for ConstGenericSmuggler<$CAP> {
                type T = $LenT;
            }
        )*
    };
}

pub trait LenUint: Add + Sub + Copy + PartialOrd + PartialEq + private::Sealed  {
    const MAX: usize;
    const ZERO: Self;
    fn from_usize(n: usize) -> Self;
    fn to_usize(self) -> usize;
}

mod private {
    pub trait Sealed {}

    impl_lenuint!(Sealed, super::LenUint, u8);
    impl_lenuint!(Sealed, super::LenUint, u16);
    impl_lenuint!(Sealed, super::LenUint, u32);
    #[cfg(target_pointer_width = "64")]
    impl_lenuint!(Sealed, super::LenUint, u64);
    impl_lenuint!(Sealed, super::LenUint, usize);
}

pub struct ConstGenericSmuggler<const CONST: usize> {}

pub trait CapToDefaultLenType {
    type T: LenUint;
}

impl_default_lentype_from_cap!(u8 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 32, 64, 100, 128, 200, 255);
impl_default_lentype_from_cap!(u16 => 256, 500, 512, 1000, 1024, 2048, 4096, 8192, 16384, 32768, 65535);
impl_default_lentype_from_cap!(u32 => 65536, 1000000, 4294967295);
impl_default_lentype_from_cap!(u64 => 18446744073709551615);

pub trait CapFitsInLenType {
    const CHECK: ();
}

impl<LenType: LenUint, const CAP: usize> CapFitsInLenType for (ConstGenericSmuggler<CAP>, LenType) {
    const CHECK: () = {
        if CAP > LenType::MAX {
            panic!("Cannot fit CAP into provided LenType");
        }
    };
}

pub type DefaultLenType<const CAP: usize> = <ConstGenericSmuggler<CAP> as CapToDefaultLenType>::T;
pub const fn check_cap_fits_in_len_type<LenType: LenUint, const CAP: usize>() {
    <(ConstGenericSmuggler<CAP>, LenType) as CapFitsInLenType>::CHECK
}
