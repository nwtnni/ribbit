use core::marker::PhantomData;
use core::num::NonZeroI128;
use core::num::NonZeroI16;
use core::num::NonZeroI32;
use core::num::NonZeroI64;
use core::num::NonZeroI8;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

pub use arbitrary_int::*;
pub use ribbit_derive::Pack;

#[cfg(feature = "atomic")]
mod atomic;
#[cfg(feature = "atomic")]
pub use atomic::Atomic;

pub type Packed<T> = <T as Pack>::Packed;
pub type Unpacked<T> = <T as Unpack>::Unpacked;

/// Marks a type that can be packed into `BITS`.
///
/// Currently supports sizes up to 128 bits.
///
/// # Safety
///
/// This trait should only be implemented by the `pack` macro.
pub unsafe trait Pack: Copy {
    type Packed: Unpack<Unpacked = Self>;

    fn pack(self) -> Self::Packed;
}

/// Marks a packed type with size `BITS`.
///
/// Currently supports sizes up to 128 bits.
///
/// # Safety
///
/// This trait should only be implemented by the `pack` macro.
///
/// Implementer must ensure:
/// - Type has size `BITS`
/// - Size and alignment of `Self::Loose` is the same as `Self`
pub unsafe trait Unpack: Copy {
    const BITS: usize;

    type Unpacked: Pack<Packed = Self>;

    #[allow(private_bounds)]
    type Loose: Loose;

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
unsafe trait Loose: Copy + Sized {
    const ZERO: Self;
}

/// Implements `const`-compatible conversions between packed and loose representations.
pub mod convert {
    use core::mem::MaybeUninit;

    use crate::Loose;
    use crate::Unpack;

    union Transmute<T: Unpack> {
        packed: T,
        loose: T::Loose,
    }

    /// Convert from a packed struct to a native integer type.
    #[inline]
    pub const fn packed_to_loose<T: Unpack>(packed: T) -> T::Loose {
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
    pub const unsafe fn loose_to_packed<T: Unpack>(loose: T::Loose) -> T {
        unsafe {
            let mut zeroed = MaybeUninit::<Transmute<T>>::zeroed();
            zeroed.write(Transmute { loose }).packed
        }
    }

    #[repr(C)]
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
            let size_from = const { core::mem::size_of::<F>() };
            let size_into = const { core::mem::size_of::<I>() };

            // Easy case: not possible to read uninitialized memory
            if size_from >= size_into {
                return Convert { from }.into;
            }

            let mut zeroed = MaybeUninit::<Convert<F, I>>::zeroed();

            // NOTE: assumes const evaluation is run with the target
            // endianness--can't find info on whether this is true
            let offset = if cfg!(target_endian = "little") {
                0
            } else {
                size_into - size_from
            };

            // Need raw pointer write (as opposed to `zeroed.write(Convert { from })`)
            // to avoid clobbering zeroed memory with uninitialized padding.
            //
            // https://google.github.io/learn_unsafe_rust/advanced_unsafety/uninitialized.html#padding
            zeroed.as_mut_ptr().byte_add(offset).cast::<F>().write(from);
            zeroed.assume_init().into
        }
    }
}

macro_rules! impl_pack {
    ($tight:ty) => {
        unsafe impl Pack for $tight {
            type Packed = Self;
            fn pack(self) -> Self::Packed {
                self
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $unsigned_loose:ty, $signed_loose:ty, $loose_bits:expr, $dollar:tt) => {
        unsafe impl Loose for $unsigned_loose {
            const ZERO: Self = 0;
        }

        unsafe impl Unpack for $unsigned_loose {
            const BITS: usize = $loose_bits;
            type Unpacked = Self;
            type Loose = Self;

            fn unpack(self) -> Self::Unpacked {
                self
            }
        }

        impl_pack!($unsigned_loose);

        unsafe impl Unpack for $signed_loose {
            const BITS: usize = $loose_bits;
            type Unpacked = Self;
            type Loose = $unsigned_loose;

            fn unpack(self) -> Self::Unpacked {
                self
            }
        }

        impl_pack!($signed_loose);

        macro_rules! $name {
            ($dollar($unsigned:ident, $signed:ident: $bits:expr),* $dollar(,)?) => {
                $dollar(
                    unsafe impl Unpack for $unsigned {
                        const BITS: usize = $bits;
                        type Unpacked = Self;
                        type Loose = $unsigned_loose;
                        fn unpack(self) -> Self::Unpacked {
                            self
                        }
                    }

                    impl_pack!($unsigned);

                    unsafe impl Unpack for $signed {
                        const BITS: usize = $bits;
                        type Unpacked = Self;
                        type Loose = $unsigned_loose;
                        fn unpack(self) -> Self::Unpacked {
                            self
                        }
                    }

                    impl_pack!($signed);
                )*
            };
        }
    };
}

impl_pack!(());

unsafe impl Unpack for () {
    const BITS: usize = 0;
    type Unpacked = Self;
    type Loose = u8;
    fn unpack(self) -> Self::Unpacked {}
}

unsafe impl<T> Pack for PhantomData<T> {
    type Packed = Self;
    fn pack(self) -> Self::Packed {
        self
    }
}

unsafe impl<T> Unpack for PhantomData<T> {
    const BITS: usize = 0;
    type Unpacked = PhantomData<T>;
    type Loose = u8;
    fn unpack(self) -> Self::Unpacked {
        self
    }
}

impl_pack!(bool);

unsafe impl Unpack for bool {
    const BITS: usize = 1;
    type Unpacked = bool;
    type Loose = u8;
    fn unpack(self) -> Self::Unpacked {
        self
    }
}

impl_impl_number!(impl_u8, u8, i8, 8, $);
impl_u8!(
    u1, i1: 1,
    u2, i2: 2,
    u3, i3: 3,
    u4, i4: 4,
    u5, i5: 5,
    u6, i6: 6,
    u7, i7: 7,
);

impl_impl_number!(impl_u16, u16, i16, 16, $);
impl_u16!(
    u9, i9: 9,
    u10, i10: 10,
    u11, i11: 11,
    u12, i12: 12,
    u13, i13: 13,
    u14, i14: 14,
    u15, i15: 15,
);

impl_impl_number!(impl_u32, u32, i32, 32, $);
impl_u32!(
    u17, i17: 17,
    u18, i18: 18,
    u19, i19: 19,
    u20, i20: 20,
    u21, i21: 21,
    u22, i22: 22,
    u23, i23: 23,
    u24, i24: 24,
    u25, i25: 25,
    u26, i26: 26,
    u27, i27: 27,
    u28, i28: 28,
    u29, i29: 29,
    u30, i30: 30,
    u31, i31: 31,
);

impl_impl_number!(impl_u64, u64, i64, 64, $);
impl_u64!(
    u33, i33: 33,
    u34, i34: 34,
    u35, i35: 35,
    u36, i36: 36,
    u37, i37: 37,
    u38, i38: 38,
    u39, i39: 39,
    u40, i40: 40,
    u41, i41: 41,
    u42, i42: 42,
    u43, i43: 43,
    u44, i44: 44,
    u45, i45: 45,
    u46, i46: 46,
    u47, i47: 47,
    u48, i48: 48,
    u49, i49: 49,
    u50, i50: 50,
    u51, i51: 51,
    u52, i52: 52,
    u53, i53: 53,
    u54, i54: 54,
    u55, i55: 55,
    u56, i56: 56,
    u57, i57: 57,
    u58, i58: 58,
    u59, i59: 59,
    u60, i60: 60,
    u61, i61: 61,
    u62, i62: 62,
    u63, i63: 63,
);

impl_impl_number!(impl_u128, u128, i128, 128, $);
impl_u128!(
    u65, i65: 65,
    u66, i66: 66,
    u67, i67: 67,
    u68, i68: 68,
    u69, i69: 69,
    u70, i70: 70,
    u71, i71: 71,
    u72, i72: 72,
    u73, i73: 73,
    u74, i74: 74,
    u75, i75: 75,
    u76, i76: 76,
    u77, i77: 77,
    u78, i78: 78,
    u79, i79: 79,
    u80, i80: 80,
    u81, i81: 81,
    u82, i82: 82,
    u83, i83: 83,
    u84, i84: 84,
    u85, i85: 85,
    u86, i86: 86,
    u87, i87: 87,
    u88, i88: 88,
    u89, i89: 89,
    u90, i90: 90,
    u91, i91: 91,
    u92, i92: 92,
    u93, i93: 93,
    u94, i94: 94,
    u95, i95: 95,
    u96, i96: 96,
    u97, i97: 97,
    u98, i98: 98,
    u99, i99: 99,
    u100, i100: 100,
    u101, i101: 101,
    u102, i102: 102,
    u103, i103: 103,
    u104, i104: 104,
    u105, i105: 105,
    u106, i106: 106,
    u107, i107: 107,
    u108, i108: 108,
    u109, i109: 109,
    u110, i110: 110,
    u111, i111: 111,
    u112, i112: 112,
    u113, i113: 113,
    u114, i114: 114,
    u115, i115: 115,
    u116, i116: 116,
    u117, i117: 117,
    u118, i118: 118,
    u119, i119: 119,
    u120, i120: 120,
    u121, i121: 121,
    u122, i122: 122,
    u123, i123: 123,
    u124, i124: 124,
    u125, i125: 125,
    u126, i126: 126,
    u127, i127: 127,
);

/// Marker trait asserting that values of this type cannot be zero.
///
/// # Safety
///
/// Implementer must guarantee that zero is not a valid bit pattern for this type.
#[allow(private_bounds)]
pub unsafe trait NonZero {}

macro_rules! impl_nonzero {
    ($unsigned:ty, $signed:ty, $loose:ty, $bits:expr) => {
        impl_pack!($unsigned);

        unsafe impl Unpack for $unsigned {
            const BITS: usize = $bits;
            type Unpacked = Self;
            type Loose = $loose;
            fn unpack(self) -> Self::Unpacked {
                self
            }
        }

        unsafe impl NonZero for $unsigned {}

        impl_pack!($signed);

        unsafe impl Unpack for $signed {
            const BITS: usize = $bits;
            type Unpacked = Self;
            type Loose = $loose;
            fn unpack(self) -> Self::Unpacked {
                self
            }
        }

        unsafe impl NonZero for $signed {}
    };
}

impl_nonzero!(NonZeroU8, NonZeroI8, u8, 8);
impl_nonzero!(NonZeroU16, NonZeroI16, u16, 16);
impl_nonzero!(NonZeroU32, NonZeroI32, u32, 32);
impl_nonzero!(NonZeroU64, NonZeroI64, u64, 64);
impl_nonzero!(NonZeroU128, NonZeroI128, u128, 128);

/// Extension trait for cheaply constructing a `ribbit::Packed<Option<T>>`
/// from the underlying integer type by reinterpreting the bits.
pub trait OptionExt {
    type Loose;
    /// # SAFETY
    ///
    /// Caller must ensure `loose` contains a valid bit pattern for `Self`.
    unsafe fn new_unchecked(loose: Self::Loose) -> Self;
}

impl<T> OptionExt for Option<T>
where
    T: Unpack + NonZero,
{
    type Loose = T::Loose;
    #[inline(always)]
    unsafe fn new_unchecked(loose: Self::Loose) -> Self {
        // SAFETY: `T::Loose` has the same size and alignment as `Self`
        core::mem::transmute_copy(&loose)
    }
}

unsafe impl<T> Pack for Option<T>
where
    T: Pack,
    T::Packed: NonZero,
{
    type Packed = Option<T::Packed>;
    fn pack(self) -> Self::Packed {
        self.map(|unpacked| unpacked.pack())
    }
}

unsafe impl<T> Unpack for Option<T>
where
    T: Unpack + NonZero,
{
    const BITS: usize = T::BITS;
    type Unpacked = Option<T::Unpacked>;
    type Loose = T::Loose;
    fn unpack(self) -> Self::Unpacked {
        self.map(|packed| packed.unpack())
    }
}

#[doc(hidden)]
pub mod private {
    pub use ::core::primitive::bool;
    pub type Unit = ();

    pub use ::core::num::NonZeroU128;
    pub use ::core::num::NonZeroU16;
    pub use ::core::num::NonZeroU32;
    pub use ::core::num::NonZeroU64;
    pub use ::core::num::NonZeroU8;

    pub use ::core::num::NonZeroI128;
    pub use ::core::num::NonZeroI16;
    pub use ::core::num::NonZeroI32;
    pub use ::core::num::NonZeroI64;
    pub use ::core::num::NonZeroI8;

    pub use ::arbitrary_int::prelude::*;
    pub use ::core::marker::PhantomData;

    pub const fn assert_nonzero<T>()
    where
        T: crate::Pack,
        T::Packed: crate::NonZero,
    {
    }

    pub const fn assert_size_eq<T>(expected: usize)
    where
        T: crate::Pack,
    {
        assert!(
            expected == <T::Packed as crate::Unpack>::BITS,
            "Annotated size does not equal actual size",
        )
    }

    pub const fn assert_size_ge<T>(expected: usize)
    where
        T: crate::Pack,
    {
        assert!(
            expected >= <T::Packed as crate::Unpack>::BITS,
            "Annotated size is less than actual size",
        )
    }
}
