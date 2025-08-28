use core::marker::PhantomData;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

pub use arbitrary_int::*;
pub use ribbit_derive::pack;

pub mod atomic;

#[macro_export]
macro_rules! Pack {
    [$unpacked:ty] => {
        <$unpacked as $crate::Pack>::Packed
    };
}

/// Marks a type that can be packed into `BITS`.
///
/// Currently supports sizes up to 128 bits.
///
/// # Safety
///
/// This trait should only be implemented by the `pack` macro.
///
/// Implementer must ensure:
/// - Type has size `BITS`
/// - Type has the same size and alignment as `Tight` and `Loose`
/// - Every valid bit pattern of type is a valid bit pattern of `Tight`.
pub unsafe trait Pack: Clone {
    /// The number of bits in the packed representation.
    const BITS: usize;

    type Packed: Unpack<Unpacked = Self>;

    #[allow(private_bounds)]
    type Loose: Loose;

    fn pack(self) -> Self::Packed;
}

pub trait Unpack: Copy {
    type Unpacked: Pack<Packed = Self>;

    fn unpack(self) -> Self::Unpacked;
}

/// Native integer type.
///
/// # Safety
///
/// Zero must be a valid bit pattern for this type.
//
// Used internally for `const`-compatible conversions between packed
// and tight types.
unsafe trait Loose: Copy {}

/// Implements `const`-compatible conversions between packed and loose representations.
pub mod convert {
    use core::mem::MaybeUninit;

    use super::Loose;
    use super::Pack;

    union Transmute<T: Pack> {
        packed: T::Packed,
        loose: T::Loose,
    }

    /// Convert from a packed struct to a native integer type.
    #[inline]
    pub const fn packed_to_loose<T: Pack>(packed: T::Packed) -> T::Loose {
        unsafe {
            let mut zeroed = MaybeUninit::<Transmute<T>>::zeroed();
            zeroed.write(Transmute { packed }).loose
        }
    }

    /// Convert from a native integer type to a packed struct.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `loose` is a valid bit pattern for type `T`.
    #[inline]
    pub const unsafe fn loose_to_packed<T: Pack>(loose: T::Loose) -> T::Packed {
        unsafe {
            let mut zeroed = MaybeUninit::<Transmute<T>>::zeroed();
            zeroed.write(Transmute { loose }).packed
        }
    }

    union Convert<F: Loose, I: Loose> {
        from: F,
        into: I,
    }

    /// Convert between two generic native integer types.
    #[inline]
    #[allow(private_bounds)]
    pub const fn loose_to_loose<F: Loose, I: Loose>(from: F) -> I {
        // SAFETY: `Loose` is only implemented for native integer types.
        unsafe {
            // Required to avoid reading uninitialized memory
            let mut zeroed = core::mem::zeroed::<Convert<F, I>>();
            zeroed.from = from;
            zeroed.into
        }
    }
}

macro_rules! impl_unpack {
    ($tight:ty) => {
        impl Unpack for $tight {
            type Unpacked = Self;
            fn unpack(self) -> Self::Unpacked {
                self
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $loose:ty, $loose_bits:expr, $dollar:tt) => {
        unsafe impl Loose for $loose {}

        unsafe impl Pack for $loose {
            const BITS: usize = $loose_bits;
            type Packed = $loose;
            type Loose = $loose;

            fn pack(self) -> Self::Packed {
                self
            }
        }

        impl_unpack!($loose);

        macro_rules! $name {
            ($dollar($tight:ident: $bits:expr),* $dollar(,)?) => {
                $dollar(
                    unsafe impl Pack for private::$tight {
                        const BITS: usize = $bits;
                        type Packed = Self;
                        type Loose = $loose;
                        fn pack(self) -> Self::Packed {
                            self
                        }
                    }

                    impl_unpack!($tight);
                )*
            };
        }
    };
}

unsafe impl Pack for () {
    const BITS: usize = 0;
    type Packed = Self;
    type Loose = u8;
    fn pack(self) -> Self::Packed {}
}

impl_unpack!(());

unsafe impl<T> Pack for PhantomData<T> {
    const BITS: usize = 0;
    type Packed = PhantomData<T>;
    type Loose = u8;
    fn pack(self) -> Self::Packed {
        self
    }
}

impl<T> Unpack for PhantomData<T> {
    type Unpacked = Self;
    fn unpack(self) -> Self::Unpacked {
        self
    }
}

unsafe impl Pack for bool {
    const BITS: usize = 1;
    type Packed = bool;
    type Loose = u8;
    fn pack(self) -> Self::Packed {
        self
    }
}

impl_unpack!(bool);

impl_impl_number!(impl_u8, u8, 8, $);
impl_u8!(
    u1: 1,
    u2: 2,
    u3: 3,
    u4: 4,
    u5: 5,
    u6: 6,
    u7: 7,
);

impl_impl_number!(impl_u16, u16, 16, $);
impl_u16!(
    u9: 9,
    u10: 10,
    u11: 11,
    u12: 12,
    u13: 13,
    u14: 14,
    u15: 15,
);

impl_impl_number!(impl_u32, u32, 32, $);
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
);

impl_impl_number!(impl_u64, u64, 64, $);
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
);

impl_impl_number!(impl_u128, u128, 128, $);
impl_u128!(
    u65: 65,
    u66: 66,
    u67: 67,
    u68: 68,
    u69: 69,
    u70: 70,
    u71: 71,
    u72: 72,
    u73: 73,
    u74: 74,
    u75: 75,
    u76: 76,
    u77: 77,
    u78: 78,
    u79: 79,
    u80: 80,
    u81: 81,
    u82: 82,
    u83: 83,
    u84: 84,
    u85: 85,
    u86: 86,
    u87: 87,
    u88: 88,
    u89: 89,
    u90: 90,
    u91: 91,
    u92: 92,
    u93: 93,
    u94: 94,
    u95: 95,
    u96: 96,
    u97: 97,
    u98: 98,
    u99: 99,
    u100: 100,
    u101: 101,
    u102: 102,
    u103: 103,
    u104: 104,
    u105: 105,
    u106: 106,
    u107: 107,
    u108: 108,
    u109: 109,
    u110: 110,
    u111: 111,
    u112: 112,
    u113: 113,
    u114: 114,
    u115: 115,
    u116: 116,
    u117: 117,
    u118: 118,
    u119: 119,
    u120: 120,
    u121: 121,
    u122: 122,
    u123: 123,
    u124: 124,
    u125: 125,
    u126: 126,
    u127: 127,
);

/// Marker trait asserting that values of this type cannot be zero.
///
/// # Safety
///
/// Sealed trait cannot be implemented outside of this crate.
///
/// Implementer must guarantee that zero is not a valid bit pattern for this type.
#[allow(private_bounds)]
pub unsafe trait NonZero: seal::Seal {}

mod seal {
    pub(super) trait Seal {}
}

macro_rules! impl_nonzero {
    ($ty:ty, $loose:ty, $bits:expr) => {
        unsafe impl Pack for $ty {
            const BITS: usize = $bits;
            type Packed = $ty;
            type Loose = $loose;
            fn pack(self) -> Self::Packed {
                self
            }
        }

        impl seal::Seal for $ty {}
        unsafe impl NonZero for $ty {}
        impl_unpack!($ty);
    };
}

impl_nonzero!(NonZeroU8, u8, 8);
impl_nonzero!(NonZeroU16, u16, 16);
impl_nonzero!(NonZeroU32, u32, 32);
impl_nonzero!(NonZeroU64, u64, 64);
impl_nonzero!(NonZeroU128, u128, 128);

unsafe impl<T> Pack for Option<T>
where
    T: Pack + NonZero,
{
    const BITS: usize = T::BITS;
    type Packed = Option<T::Packed>;
    type Loose = T::Loose;
    fn pack(self) -> Self::Packed {
        self.map(|unpacked| unpacked.pack())
    }
}

impl<T> Unpack for Option<T>
where
    T: Unpack,
    <T as Unpack>::Unpacked: NonZero,
{
    type Unpacked = Option<T::Unpacked>;
    fn unpack(self) -> Self::Unpacked {
        self.map(|packed| packed.unpack())
    }
}

#[doc(hidden)]
#[rustfmt::skip]
pub mod private {
    pub use ::core::primitive::bool;
    pub type Unit = ();

    pub use ::core::primitive::u8;
    pub use ::core::primitive::u16;
    pub use ::core::primitive::u32;
    pub use ::core::primitive::u64;
    pub use ::core::primitive::u128;

    pub use ::core::num::NonZeroU8;
    pub use ::core::num::NonZeroU16;
    pub use ::core::num::NonZeroU32;
    pub use ::core::num::NonZeroU64;
    pub use ::core::num::NonZeroU128;

    pub use ::arbitrary_int::*;
    pub use ::static_assertions::assert_impl_all;
    pub use ::const_panic::concat_assert;
    pub use ::core::marker::PhantomData;
}
