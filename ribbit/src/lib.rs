use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;
use core::ops::Shl;

use arbitrary_int::Number;
pub use ribbit_derive::pack;

pub trait Packed {
    type Repr: Number;
}

pub unsafe trait NonZero {}

unsafe impl NonZero for NonZeroU8 {}
unsafe impl NonZero for NonZeroU16 {}
unsafe impl NonZero for NonZeroU32 {}
unsafe impl NonZero for NonZeroU64 {}

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
