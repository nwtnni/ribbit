#![no_std]

pub use core::marker::PhantomData;
pub use core::num::NonZeroI128;
pub use core::num::NonZeroI16;
pub use core::num::NonZeroI32;
pub use core::num::NonZeroI64;
pub use core::num::NonZeroI8;
pub use core::num::NonZeroU128;
pub use core::num::NonZeroU16;
pub use core::num::NonZeroU32;
pub use core::num::NonZeroU64;
pub use core::num::NonZeroU8;
pub use core::primitive::bool;
pub use core::primitive::u128;
pub use core::primitive::u16;
pub use core::primitive::u32;
pub use core::primitive::u64;
pub use core::primitive::u8;
pub type Unit = ();

pub use arbitrary_int::i1;
pub use arbitrary_int::i10;
pub use arbitrary_int::i100;
pub use arbitrary_int::i101;
pub use arbitrary_int::i102;
pub use arbitrary_int::i103;
pub use arbitrary_int::i104;
pub use arbitrary_int::i105;
pub use arbitrary_int::i106;
pub use arbitrary_int::i107;
pub use arbitrary_int::i108;
pub use arbitrary_int::i109;
pub use arbitrary_int::i11;
pub use arbitrary_int::i110;
pub use arbitrary_int::i111;
pub use arbitrary_int::i112;
pub use arbitrary_int::i113;
pub use arbitrary_int::i114;
pub use arbitrary_int::i115;
pub use arbitrary_int::i116;
pub use arbitrary_int::i117;
pub use arbitrary_int::i118;
pub use arbitrary_int::i119;
pub use arbitrary_int::i12;
pub use arbitrary_int::i120;
pub use arbitrary_int::i121;
pub use arbitrary_int::i122;
pub use arbitrary_int::i123;
pub use arbitrary_int::i124;
pub use arbitrary_int::i125;
pub use arbitrary_int::i126;
pub use arbitrary_int::i127;
pub use arbitrary_int::i13;
pub use arbitrary_int::i14;
pub use arbitrary_int::i15;
pub use arbitrary_int::i17;
pub use arbitrary_int::i18;
pub use arbitrary_int::i19;
pub use arbitrary_int::i2;
pub use arbitrary_int::i20;
pub use arbitrary_int::i21;
pub use arbitrary_int::i22;
pub use arbitrary_int::i23;
pub use arbitrary_int::i24;
pub use arbitrary_int::i25;
pub use arbitrary_int::i26;
pub use arbitrary_int::i27;
pub use arbitrary_int::i28;
pub use arbitrary_int::i29;
pub use arbitrary_int::i3;
pub use arbitrary_int::i30;
pub use arbitrary_int::i31;
pub use arbitrary_int::i33;
pub use arbitrary_int::i34;
pub use arbitrary_int::i35;
pub use arbitrary_int::i36;
pub use arbitrary_int::i37;
pub use arbitrary_int::i38;
pub use arbitrary_int::i39;
pub use arbitrary_int::i4;
pub use arbitrary_int::i40;
pub use arbitrary_int::i41;
pub use arbitrary_int::i42;
pub use arbitrary_int::i43;
pub use arbitrary_int::i44;
pub use arbitrary_int::i45;
pub use arbitrary_int::i46;
pub use arbitrary_int::i47;
pub use arbitrary_int::i48;
pub use arbitrary_int::i49;
pub use arbitrary_int::i5;
pub use arbitrary_int::i50;
pub use arbitrary_int::i51;
pub use arbitrary_int::i52;
pub use arbitrary_int::i53;
pub use arbitrary_int::i54;
pub use arbitrary_int::i55;
pub use arbitrary_int::i56;
pub use arbitrary_int::i57;
pub use arbitrary_int::i58;
pub use arbitrary_int::i59;
pub use arbitrary_int::i6;
pub use arbitrary_int::i60;
pub use arbitrary_int::i61;
pub use arbitrary_int::i62;
pub use arbitrary_int::i63;
pub use arbitrary_int::i65;
pub use arbitrary_int::i66;
pub use arbitrary_int::i67;
pub use arbitrary_int::i68;
pub use arbitrary_int::i69;
pub use arbitrary_int::i7;
pub use arbitrary_int::i70;
pub use arbitrary_int::i71;
pub use arbitrary_int::i72;
pub use arbitrary_int::i73;
pub use arbitrary_int::i74;
pub use arbitrary_int::i75;
pub use arbitrary_int::i76;
pub use arbitrary_int::i77;
pub use arbitrary_int::i78;
pub use arbitrary_int::i79;
pub use arbitrary_int::i80;
pub use arbitrary_int::i81;
pub use arbitrary_int::i82;
pub use arbitrary_int::i83;
pub use arbitrary_int::i84;
pub use arbitrary_int::i85;
pub use arbitrary_int::i86;
pub use arbitrary_int::i87;
pub use arbitrary_int::i88;
pub use arbitrary_int::i89;
pub use arbitrary_int::i9;
pub use arbitrary_int::i90;
pub use arbitrary_int::i91;
pub use arbitrary_int::i92;
pub use arbitrary_int::i93;
pub use arbitrary_int::i94;
pub use arbitrary_int::i95;
pub use arbitrary_int::i96;
pub use arbitrary_int::i97;
pub use arbitrary_int::i98;
pub use arbitrary_int::i99;
pub use arbitrary_int::u1;
pub use arbitrary_int::u10;
pub use arbitrary_int::u100;
pub use arbitrary_int::u101;
pub use arbitrary_int::u102;
pub use arbitrary_int::u103;
pub use arbitrary_int::u104;
pub use arbitrary_int::u105;
pub use arbitrary_int::u106;
pub use arbitrary_int::u107;
pub use arbitrary_int::u108;
pub use arbitrary_int::u109;
pub use arbitrary_int::u11;
pub use arbitrary_int::u110;
pub use arbitrary_int::u111;
pub use arbitrary_int::u112;
pub use arbitrary_int::u113;
pub use arbitrary_int::u114;
pub use arbitrary_int::u115;
pub use arbitrary_int::u116;
pub use arbitrary_int::u117;
pub use arbitrary_int::u118;
pub use arbitrary_int::u119;
pub use arbitrary_int::u12;
pub use arbitrary_int::u120;
pub use arbitrary_int::u121;
pub use arbitrary_int::u122;
pub use arbitrary_int::u123;
pub use arbitrary_int::u124;
pub use arbitrary_int::u125;
pub use arbitrary_int::u126;
pub use arbitrary_int::u127;
pub use arbitrary_int::u13;
pub use arbitrary_int::u14;
pub use arbitrary_int::u15;
pub use arbitrary_int::u17;
pub use arbitrary_int::u18;
pub use arbitrary_int::u19;
pub use arbitrary_int::u2;
pub use arbitrary_int::u20;
pub use arbitrary_int::u21;
pub use arbitrary_int::u22;
pub use arbitrary_int::u23;
pub use arbitrary_int::u24;
pub use arbitrary_int::u25;
pub use arbitrary_int::u26;
pub use arbitrary_int::u27;
pub use arbitrary_int::u28;
pub use arbitrary_int::u29;
pub use arbitrary_int::u3;
pub use arbitrary_int::u30;
pub use arbitrary_int::u31;
pub use arbitrary_int::u33;
pub use arbitrary_int::u34;
pub use arbitrary_int::u35;
pub use arbitrary_int::u36;
pub use arbitrary_int::u37;
pub use arbitrary_int::u38;
pub use arbitrary_int::u39;
pub use arbitrary_int::u4;
pub use arbitrary_int::u40;
pub use arbitrary_int::u41;
pub use arbitrary_int::u42;
pub use arbitrary_int::u43;
pub use arbitrary_int::u44;
pub use arbitrary_int::u45;
pub use arbitrary_int::u46;
pub use arbitrary_int::u47;
pub use arbitrary_int::u48;
pub use arbitrary_int::u49;
pub use arbitrary_int::u5;
pub use arbitrary_int::u50;
pub use arbitrary_int::u51;
pub use arbitrary_int::u52;
pub use arbitrary_int::u53;
pub use arbitrary_int::u54;
pub use arbitrary_int::u55;
pub use arbitrary_int::u56;
pub use arbitrary_int::u57;
pub use arbitrary_int::u58;
pub use arbitrary_int::u59;
pub use arbitrary_int::u6;
pub use arbitrary_int::u60;
pub use arbitrary_int::u61;
pub use arbitrary_int::u62;
pub use arbitrary_int::u63;
pub use arbitrary_int::u65;
pub use arbitrary_int::u66;
pub use arbitrary_int::u67;
pub use arbitrary_int::u68;
pub use arbitrary_int::u69;
pub use arbitrary_int::u7;
pub use arbitrary_int::u70;
pub use arbitrary_int::u71;
pub use arbitrary_int::u72;
pub use arbitrary_int::u73;
pub use arbitrary_int::u74;
pub use arbitrary_int::u75;
pub use arbitrary_int::u76;
pub use arbitrary_int::u77;
pub use arbitrary_int::u78;
pub use arbitrary_int::u79;
pub use arbitrary_int::u80;
pub use arbitrary_int::u81;
pub use arbitrary_int::u82;
pub use arbitrary_int::u83;
pub use arbitrary_int::u84;
pub use arbitrary_int::u85;
pub use arbitrary_int::u86;
pub use arbitrary_int::u87;
pub use arbitrary_int::u88;
pub use arbitrary_int::u89;
pub use arbitrary_int::u9;
pub use arbitrary_int::u90;
pub use arbitrary_int::u91;
pub use arbitrary_int::u92;
pub use arbitrary_int::u93;
pub use arbitrary_int::u94;
pub use arbitrary_int::u95;
pub use arbitrary_int::u96;
pub use arbitrary_int::u97;
pub use arbitrary_int::u98;
pub use arbitrary_int::u99;

pub use ribbit_derive::Pack;

#[cfg(feature = "atomic")]
pub mod atomic;
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

    #[expect(private_bounds)]
    type Loose: Loose;

    type Raw: Unpack<Unpacked = Self::Raw, Loose = Self::Loose, Raw = Self::Raw>;

    fn unpack(self) -> Self::Unpacked;

    fn into_raw(self) -> Self::Raw;

    unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self;
}

/// Native integer type.
///
/// # Safety
///
/// Zero must be a valid bit pattern for this type.
//
// Used internally for `const`-compatible conversions between packed
// and tight types.
unsafe trait Loose: Copy + Sized + Unpack<Unpacked = Self, Loose = Self, Raw = Self> {}

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

            #[inline]
            fn pack(self) -> Self::Packed {
                self
            }
        }
    };
}

macro_rules! impl_unpack {
    ($tight:ty, $bits:expr, $loose:ty) => {
        unsafe impl Unpack for $tight {
            const BITS: usize = $bits;
            type Unpacked = Self;
            type Loose = $loose;
            type Raw = Self;

            #[inline]
            fn unpack(self) -> Self::Unpacked {
                self
            }

            #[inline]
            fn into_raw(self) -> Self::Raw {
                self
            }

            #[inline]
            unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self {
                raw
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! impl_impl_number {
    ($name:ident, $unsigned_loose:ty, $signed_loose:ty, $loose_bits:expr, $dollar:tt) => {
        unsafe impl Loose for $unsigned_loose {}

        impl_pack!($unsigned_loose);
        impl_unpack!($unsigned_loose, $loose_bits, $unsigned_loose);

        impl_pack!($signed_loose);
        impl_unpack!($signed_loose, $loose_bits, $unsigned_loose);

        macro_rules! $name {
            ($dollar($unsigned:ident, $signed:ident: $bits:expr),* $dollar(,)?) => {
                $dollar(
                    impl_pack!($unsigned);
                    impl_unpack!($unsigned, $bits, $unsigned_loose);

                    impl_pack!($signed);
                    impl_unpack!($signed, $bits, $unsigned_loose);
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
    type Raw = ();

    #[inline]
    fn unpack(self) -> Self::Unpacked {}

    #[inline]
    fn into_raw(self) -> Self::Raw {}

    #[inline]
    unsafe fn from_raw_unchecked(_: Self::Raw) -> Self {}
}

unsafe impl<T> Pack for PhantomData<T> {
    type Packed = Self;

    #[inline]
    fn pack(self) -> Self::Packed {
        self
    }
}

unsafe impl<T> Unpack for PhantomData<T> {
    const BITS: usize = 0;
    type Unpacked = PhantomData<T>;
    type Loose = u8;
    type Raw = Self;

    #[inline]
    fn unpack(self) -> Self::Unpacked {
        self
    }

    #[inline]
    fn into_raw(self) -> Self::Raw {
        self
    }

    #[inline]
    unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self {
        raw
    }
}

impl_pack!(bool);

unsafe impl Unpack for bool {
    const BITS: usize = 1;
    type Unpacked = bool;
    type Loose = u8;
    type Raw = bool;

    #[inline]
    fn unpack(self) -> Self::Unpacked {
        self
    }

    #[inline]
    fn into_raw(self) -> Self::Raw {
        self
    }

    #[inline]
    unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self {
        raw
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
unsafe trait NonZero {}

macro_rules! impl_nonzero {
    ($unsigned:ty, $signed:ty, $loose:ty, $bits:expr) => {
        impl_pack!($unsigned);
        impl_unpack!($unsigned, $bits, $loose);
        unsafe impl NonZero for $unsigned {}

        impl_pack!($signed);
        impl_unpack!($signed, $bits, $loose);
        unsafe impl NonZero for $signed {}
    };
}

impl_nonzero!(NonZeroU8, NonZeroI8, u8, 8);
impl_nonzero!(NonZeroU16, NonZeroI16, u16, 16);
impl_nonzero!(NonZeroU32, NonZeroI32, u32, 32);
impl_nonzero!(NonZeroU64, NonZeroI64, u64, 64);
impl_nonzero!(NonZeroU128, NonZeroI128, u128, 128);

unsafe impl<T> Pack for Option<T>
where
    T: Pack,
    <T::Packed as Unpack>::Raw: NonZero,
{
    type Packed = Option<T::Packed>;
    fn pack(self) -> Self::Packed {
        self.map(|unpacked| unpacked.pack())
    }
}

unsafe impl<T> Unpack for Option<T>
where
    T: Unpack<Raw: NonZero>,
{
    const BITS: usize = T::BITS;
    type Unpacked = Option<T::Unpacked>;
    type Loose = T::Loose;
    type Raw = T::Loose;

    #[inline]
    fn unpack(self) -> Self::Unpacked {
        self.map(|packed| packed.unpack())
    }

    #[inline]
    fn into_raw(self) -> Self::Raw {
        const {
            assert!(size_of::<Self>() == size_of::<T>());
            assert!(align_of::<Self>() == align_of::<T>());
        }

        // SAFETY: `Self::Raw` implements `NonZero`
        unsafe { core::mem::transmute_copy(&self) }
    }

    #[inline]
    unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self {
        const {
            assert!(size_of::<Self>() == size_of::<T>());
            assert!(align_of::<Self>() == align_of::<T>());
        }

        core::mem::transmute_copy(&raw)
    }
}

#[doc(hidden)]
pub mod private {
    #[expect(private_bounds)]
    pub const fn assert_nonzero<T>()
    where
        T: crate::Pack,
        <T::Packed as crate::Unpack>::Raw: crate::NonZero,
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
