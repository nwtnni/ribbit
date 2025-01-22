use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

pub use ribbit_derive::pack;

/// Marks a type that can be packed into `BITS`.
///
/// Currently supports sizes up to 64 bits.
pub unsafe trait Pack: Copy + Sized {
    /// The number of bits in the packed representation.
    const BITS: usize;

    #[allow(private_bounds)]
    type Tight: Copy + Tight;

    #[allow(private_bounds)]
    type Loose: Copy + Loose;
}

trait Loose {}
trait Tight {}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $loose:ty, $dollar:tt) => {
        impl Loose for $loose {}

        macro_rules! $name {
            ($dollar($ty:ident: $bits:expr),* $dollar(,)?) => {
                $dollar(
                    unsafe impl Pack for private::$ty {
                        const BITS: usize = $bits;
                        type Tight = private::$ty;
                        type Loose = $loose;
                    }

                    impl Tight for private::$ty {}
                )*
            };
        }
    };
}

unsafe impl Pack for () {
    const BITS: usize = 0;
    type Tight = ();
    type Loose = ();
}

impl Loose for () {}
impl Tight for () {}

unsafe impl Pack for bool {
    const BITS: usize = 1;
    type Tight = bool;
    type Loose = u8;
}

impl Tight for bool {}

impl_impl_number!(impl_u8, u8, $);
impl_u8!(
    u1: 1,
    u2: 2,
    u3: 3,
    u4: 4,
    u5: 5,
    u6: 6,
    u7: 7,
    u8: 8,
);

impl_impl_number!(impl_u16, u16, $);
impl_u16!(
    u9: 9,
    u10: 10,
    u11: 11,
    u12: 12,
    u13: 13,
    u14: 14,
    u15: 15,
    u16: 16,
);

impl_impl_number!(impl_u32, u32, $);
impl_u32!(
    u17: 17,
    u18: 18,
    u19: 19,
    u20: 20,
    u21: 21,
    u22: 22,
    u23: 23,
    u24: 24,
    u25: 25,
    u26: 26,
    u27: 27,
    u28: 28,
    u29: 29,
    u30: 30,
    u31: 31,
    u32: 32,
);

impl_impl_number!(impl_u64, u64, $);
impl_u64!(
    u33: 33,
    u34: 34,
    u35: 35,
    u36: 36,
    u37: 37,
    u38: 38,
    u39: 39,
    u40: 40,
    u41: 41,
    u42: 42,
    u43: 43,
    u44: 44,
    u45: 45,
    u46: 46,
    u47: 47,
    u48: 48,
    u49: 49,
    u50: 50,
    u51: 51,
    u52: 52,
    u53: 53,
    u54: 54,
    u55: 55,
    u56: 56,
    u57: 57,
    u58: 58,
    u59: 59,
    u60: 60,
    u61: 61,
    u62: 62,
    u63: 63,
    u64: 64,
);

#[allow(private_bounds)]
pub trait NonZero: seal::Seal {}

mod seal {
    pub(super) trait Seal {}
}

macro_rules! impl_nonzero {
    ($ty:ty, $loose:ty, $bits:expr) => {
        unsafe impl Pack for $ty {
            const BITS: usize = $bits;
            type Tight = $ty;
            type Loose = $loose;
        }

        impl Tight for $ty {}
        impl seal::Seal for $ty {}
        impl NonZero for $ty {}
    };
}

impl_nonzero!(NonZeroU8, u8, 8);
impl_nonzero!(NonZeroU16, u16, 16);
impl_nonzero!(NonZeroU32, u32, 32);
impl_nonzero!(NonZeroU64, u64, 64);

impl<T> Tight for Option<T> where T: Tight + NonZero {}

unsafe impl<T> Pack for Option<T>
where
    T: Pack,
    T::Tight: Tight + NonZero,
{
    const BITS: usize = T::BITS;
    type Tight = Option<T::Tight>;
    type Loose = T::Loose;
}

#[doc(hidden)]
#[rustfmt::skip]
pub mod private {
    pub use ::core::primitive::bool;
    pub type Unit = ();

    pub use ::arbitrary_int::Number;
    pub use ::arbitrary_int::u1;
    pub use ::arbitrary_int::u2;
    pub use ::arbitrary_int::u3;
    pub use ::arbitrary_int::u4;
    pub use ::arbitrary_int::u5;
    pub use ::arbitrary_int::u6;
    pub use ::arbitrary_int::u7;
    pub use ::core::primitive::u8;
    pub use ::arbitrary_int::u9;
    pub use ::arbitrary_int::u10;
    pub use ::arbitrary_int::u11;
    pub use ::arbitrary_int::u12;
    pub use ::arbitrary_int::u13;
    pub use ::arbitrary_int::u14;
    pub use ::arbitrary_int::u15;
    pub use ::core::primitive::u16;
    pub use ::arbitrary_int::u17;
    pub use ::arbitrary_int::u18;
    pub use ::arbitrary_int::u19;
    pub use ::arbitrary_int::u20;
    pub use ::arbitrary_int::u21;
    pub use ::arbitrary_int::u22;
    pub use ::arbitrary_int::u23;
    pub use ::arbitrary_int::u24;
    pub use ::arbitrary_int::u25;
    pub use ::arbitrary_int::u26;
    pub use ::arbitrary_int::u27;
    pub use ::arbitrary_int::u28;
    pub use ::arbitrary_int::u29;
    pub use ::arbitrary_int::u30;
    pub use ::arbitrary_int::u31;
    pub use ::core::primitive::u32;
    pub use ::arbitrary_int::u33;
    pub use ::arbitrary_int::u34;
    pub use ::arbitrary_int::u35;
    pub use ::arbitrary_int::u36;
    pub use ::arbitrary_int::u37;
    pub use ::arbitrary_int::u38;
    pub use ::arbitrary_int::u39;
    pub use ::arbitrary_int::u40;
    pub use ::arbitrary_int::u41;
    pub use ::arbitrary_int::u42;
    pub use ::arbitrary_int::u43;
    pub use ::arbitrary_int::u44;
    pub use ::arbitrary_int::u45;
    pub use ::arbitrary_int::u46;
    pub use ::arbitrary_int::u47;
    pub use ::arbitrary_int::u48;
    pub use ::arbitrary_int::u49;
    pub use ::arbitrary_int::u50;
    pub use ::arbitrary_int::u51;
    pub use ::arbitrary_int::u52;
    pub use ::arbitrary_int::u53;
    pub use ::arbitrary_int::u54;
    pub use ::arbitrary_int::u55;
    pub use ::arbitrary_int::u56;
    pub use ::arbitrary_int::u57;
    pub use ::arbitrary_int::u58;
    pub use ::arbitrary_int::u59;
    pub use ::arbitrary_int::u60;
    pub use ::arbitrary_int::u61;
    pub use ::arbitrary_int::u62;
    pub use ::arbitrary_int::u63;
    pub use ::core::primitive::u64;

    pub use ::core::num::NonZeroU8;
    pub use ::core::num::NonZeroU16;
    pub use ::core::num::NonZeroU32;
    pub use ::core::num::NonZeroU64;

    pub use ::static_assertions::assert_impl_all;
    pub use ::core::marker::PhantomData;

    use crate::Pack;

    union Transmute<T: Pack> {
        value: T,
        loose: T::Loose,
    }

    const fn assert_layout<T: Pack>() {
        const {
            assert!(
                core::mem::size_of::<T>() == core::mem::size_of::<T::Tight>()
                && core::mem::size_of::<T>() == core::mem::size_of::<T::Loose>()
            );

            assert!(
                core::mem::align_of::<T>() == core::mem::align_of::<T::Tight>()
                && core::mem::align_of::<T>() == core::mem::align_of::<T::Loose>()
            );
        }
    }

    pub const fn pack<T: Pack>(value: T) -> T::Loose {
        const { assert_layout::<T>() }
        unsafe { Transmute { value }.loose }
    }

    pub const unsafe fn unpack<T: Pack>(loose: T::Loose) -> T {
        const { assert_layout::<T>() }
         Transmute { loose }.value
    }
}
