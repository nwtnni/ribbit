use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

pub use ribbit_derive::pack;

pub unsafe trait Pack: Copy + Sized {
    type Repr: Number;
}

union Transmute<T: Pack> {
    r#in: T,
    out: T::Repr,
}

pub const fn pack<T: Pack>(value: T) -> T::Repr {
    const {
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<T::Repr>());
        assert!(core::mem::align_of::<T>() == core::mem::align_of::<T::Repr>());
    }

    unsafe { Transmute { r#in: value }.out }
}

pub const fn unpack<T: Pack>(value: T::Repr) -> T {
    const {
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<T::Repr>());
        assert!(core::mem::align_of::<T>() == core::mem::align_of::<T::Repr>());
    }

    unsafe { Transmute { out: value }.r#in }
}

pub trait Number: Copy + Sized {
    type Repr;
    const BITS: usize;
    const MIN: Self;
    const MAX: Self;

    fn new(value: Self::Repr) -> Self;
    fn value(self) -> Self::Repr;
}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $repr:ty, $dollar:tt) => {
        macro_rules! $name {
            ($dollar($ty:ident),*) => {
                $dollar(
                    unsafe impl Pack for private::$ty {
                        type Repr = Self;
                    }

                    impl Number for private::$ty {
                        type Repr = $repr;
                        const BITS: usize = <private::$ty as arbitrary_int::Number>::BITS;
                        const MIN: Self = <private::$ty as arbitrary_int::Number>::MIN;
                        const MAX: Self = <private::$ty as arbitrary_int::Number>::MAX;
                        fn new(value: Self::Repr) -> Self {
                            <private::$ty as arbitrary_int::Number>::new(value)
                        }
                        fn value(self) -> Self::Repr {
                            <private::$ty as arbitrary_int::Number>::value(self)
                        }
                    }
                )*
            };
        }
    };
}

impl_impl_number!(impl_u8, u8, $);
impl_u8!(u1, u2, u3, u4, u5, u6, u7, u8);

impl_impl_number!(impl_u16, u16, $);
impl_u16!(u9, u10, u11, u12, u13, u14, u15);

impl_impl_number!(impl_u32, u32, $);
impl_u32!(u17, u18, u19, u20, u21, u22, u23, u24, u25, u26, u27, u28, u29, u30, u31, u32);

impl_impl_number!(impl_u64, u64, $);
impl_u64!(
    u33, u34, u35, u36, u37, u38, u39, u40, u41, u42, u43, u44, u45, u46, u47, u48, u49, u50, u51,
    u52, u53, u54, u55, u56, u57, u58, u59, u60, u61, u62, u63, u64
);

macro_rules! impl_nonzero {
    ($ty:ty, $bits:expr) => {
        unsafe impl Pack for $ty {
            type Repr = $ty;
        }

        unsafe impl NonZero for $ty {}

        impl Number for $ty {
            type Repr = $ty;
            const BITS: usize = $bits;
            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;
            fn new(value: Self::Repr) -> Self {
                value
            }
            fn value(self) -> Self::Repr {
                self
            }
        }
    };
}

impl_nonzero!(NonZeroU8, 8);
impl_nonzero!(NonZeroU16, 16);
impl_nonzero!(NonZeroU32, 32);
impl_nonzero!(NonZeroU64, 64);

unsafe impl<T> Pack for Option<T>
where
    T: Pack + NonZero,
{
    type Repr = <T as Pack>::Repr;
}

pub unsafe trait NonZero {}

#[doc(hidden)]
#[rustfmt::skip]
pub mod private {
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
}
