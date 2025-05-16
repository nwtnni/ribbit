use core::marker::PhantomData;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

pub use arbitrary_int::*;
pub use ribbit_derive::pack;

#[macro_export]
macro_rules! unpack {
    ($packed:ty) => {
        <$packed as $crate::Pack>::Unpack
    };
}

/// Marks a type that can be packed into `BITS`.
///
/// Currently supports sizes up to 64 bits.
///
/// # Safety
///
/// This trait should only be implemented by the `pack` macro.
///
/// Implementer must ensure:
/// - Type has size `BITS`
/// - Type has the same size and alignment as `Tight` and `Loose`
/// - Every valid bit pattern of type is a valid bit pattern of `Tight`.
pub unsafe trait Pack: Copy + Sized {
    /// The number of bits in the packed representation.
    const BITS: usize;

    type Unpack;

    #[allow(private_bounds)]
    type Tight: Tight;

    #[allow(private_bounds)]
    type Loose: Loose;

    fn to_loose(&self) -> Self::Loose {
        convert::packed_to_loose(*self)
    }

    fn to_tight(&self) -> Self::Tight {
        convert::packed_to_tight(*self)
    }

    fn unpack(&self) -> Self::Unpack;
}

/// Native integer type, or non-native integer type with stronger guarantees
/// (e.g. `NonZero`, arbitrary-sized integer).
//
// Used internally as the backing representation of a packed type,
// so that the compiler can take advantage of the type's guarantees.
trait Tight: Copy {}

/// Native integer type.
///
/// # Safety
///
/// Zero must be a valid bit pattern for this type.
//
// Used internally for `const`-compatible conversions between packed
// and tight types.
unsafe trait Loose: Copy {}

impl<T: Loose> Tight for T {}

/// Implements `const`-compatible conversions between packed, loose, and tight
/// representations.
pub mod convert {
    use super::Loose;
    use super::Pack;

    union Transmute<T: Pack> {
        value: T,
        loose: T::Loose,
        tight: T::Tight,
    }

    /// Convert from a packed struct to a native integer type.
    #[inline]
    pub const fn packed_to_loose<T: Pack>(value: T) -> T::Loose {
        const { assert_layout::<T>() }
        unsafe { Transmute { value }.loose }
    }

    /// Convert from a packed struct to an integer type.
    #[inline]
    pub const fn packed_to_tight<T: Pack>(value: T) -> T::Tight {
        const { assert_layout::<T>() }
        unsafe { Transmute { value }.tight }
    }

    /// Convert from a native integer type to a packed struct.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `loose` is a valid bit pattern for type `T`.
    #[inline]
    pub const unsafe fn loose_to_packed<T: Pack>(loose: T::Loose) -> T {
        const { assert_layout::<T>() }
        Transmute { loose }.value
    }

    /// Convert from an integer type to a packed struct.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that `loose` is a valid bit pattern for type `T`.
    #[inline]
    pub const unsafe fn tight_to_packed<T: Pack>(tight: T::Tight) -> T {
        const { assert_layout::<T>() }
        Transmute { tight }.value
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

    // Sanity check for size and alignment at compile time.
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
}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $loose:ty, $loose_bits:expr, $dollar:tt) => {
        unsafe impl Loose for $loose {}

        unsafe impl Pack for $loose {
            const BITS: usize = $loose_bits;
            type Unpack = $loose;
            type Tight = $loose;
            type Loose = $loose;

            fn unpack(&self) -> Self::Unpack {
                *self
            }
        }

        macro_rules! $name {
            ($dollar($tight:ident: $bits:expr),* $dollar(,)?) => {
                $dollar(
                    impl Tight for private::$tight {}

                    unsafe impl Pack for private::$tight {
                        const BITS: usize = $bits;
                        type Unpack = private::$tight;
                        type Tight = private::$tight;
                        type Loose = $loose;
                        fn unpack(&self) -> Self::Unpack {
                            *self
                        }
                    }
                )*
            };
        }
    };
}

unsafe impl Pack for () {
    const BITS: usize = 0;
    type Unpack = ();
    type Tight = ();
    type Loose = ();
    fn unpack(&self) -> Self::Unpack {}
}

unsafe impl Loose for () {}

unsafe impl<T> Pack for PhantomData<T> {
    const BITS: usize = 0;
    type Unpack = PhantomData<T>;
    type Tight = ();
    type Loose = ();
    fn unpack(&self) -> Self::Unpack {
        *self
    }
}

unsafe impl Pack for bool {
    const BITS: usize = 1;
    type Unpack = bool;
    type Tight = bool;
    type Loose = u8;
    fn unpack(&self) -> Self::Unpack {
        *self
    }
}

unsafe impl Loose for bool {}

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
            type Unpack = $ty;
            type Tight = $ty;
            type Loose = $loose;
            fn unpack(&self) -> Self::Unpack {
                *self
            }
        }

        impl Tight for $ty {}
        impl seal::Seal for $ty {}
        unsafe impl NonZero for $ty {}
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
    type Unpack = Option<T::Unpack>;
    type Tight = Option<T::Tight>;
    type Loose = T::Loose;
    fn unpack(&self) -> Self::Unpack {
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

    pub use ::core::num::NonZeroU8;
    pub use ::core::num::NonZeroU16;
    pub use ::core::num::NonZeroU32;
    pub use ::core::num::NonZeroU64;

    pub use ::arbitrary_int::*;
    pub use ::static_assertions::assert_impl_all;
    pub use ::const_panic::concat_assert;
    pub use ::core::marker::PhantomData;
}
