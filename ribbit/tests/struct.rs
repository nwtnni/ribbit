use core::num::NonZeroI32;
use core::num::NonZeroI8;
use core::num::NonZeroU16;

use ribbit::i24;
use ribbit::i40;
use ribbit::u1;
use ribbit::u2;
use ribbit::u24;
use ribbit::u40;
use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct Smoke {
    a: u32,
    b: u32,
}

#[test]
fn smoke() {
    let h = Smoke {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.into_raw(), 0xbeef_dead_dead_beef);
    assert_eq!(h.a(), 0xdead_beef);
    assert_eq!(h.b(), 0xbeef_dead);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct Arbitrary {
    a: u40,
    b: u24,
}

#[test]
fn arbitrary_field() {
    let h = Arbitrary {
        a: u40::new(0xad_dead_beef),
        b: u24::new(0xbe_efde),
    }
    .pack();

    assert_eq!(h.into_raw(), 0xbeef_dead_dead_beef);
    assert_eq!(h.a().value(), 0xad_dead_beef);
    assert_eq!(h.b().value(), 0xbe_efde);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 9)]
struct Unaligned {
    a: ribbit::u1,
    b: u8,
}

#[test]
fn unaligned_field() {
    let h = Unaligned {
        a: u1::new(1),
        b: 0b10101010,
    }
    .pack();
    assert_eq!(h.a().value(), 0b1);
    assert_eq!(h.b(), 0b10101010);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 2)]
struct AdjacentBit {
    a: ribbit::u1,
    b: ribbit::u1,
}

#[test]
fn adjacent_bits() {
    let h = AdjacentBit {
        a: u1::new(0),
        b: u1::new(0),
    }
    .pack();

    assert_eq!(h.a().value(), 0b0);
    assert_eq!(h.b().value(), 0b0);

    let i = h.with_a(true.into());

    assert_eq!(i.a().value(), 0b1);
    assert_eq!(i.b().value(), 0b0);

    let j = i.with_b(true.into());

    assert_eq!(j.a().value(), 0b1);
    assert_eq!(j.b().value(), 0b1);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 2)]
struct TwoBit {
    value: u2,
}

#[test]
fn two_bit() {
    let c = TwoBit { value: u2::new(0) }.pack();

    assert_eq!(c.value(), u2::new(0b00));

    let c = c.with_value(u2::new(0b01));
    assert_eq!(c.value(), u2::new(0b01));

    let c = c.with_value(u2::new(0b10));
    assert_eq!(c.value(), u2::new(0b10));
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 16, nonzero)]
struct NonZero {
    nonzero: ribbit::NonZeroU16,
}

#[test]
fn nonzero_from_raw() {
    let non_zero = unsafe { ribbit::Packed::<Option<NonZero>>::from_raw_unchecked(15) };
    assert_eq!(
        non_zero.unpack(),
        Some(NonZero {
            nonzero: NonZeroU16::new(15).unwrap()
        })
    );

    let zero = unsafe { ribbit::Packed::<Option<NonZero>>::from_raw_unchecked(0) };
    assert_eq!(zero.unpack(), None);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 18)]
struct ExplicitOffset {
    #[ribbit(offset = 2)]
    a: ribbit::NonZeroU16,
    b: ribbit::u2,
}

#[test]
fn explicit_offset() {
    let mix = ExplicitOffset {
        a: NonZeroU16::new(55).unwrap(),
        b: u2::new(3),
    }
    .pack();
    assert_eq!(mix.a().get(), 55);
    assert_eq!(mix.b().value(), 3);

    let mix = mix.with_a(NonZeroU16::new(999).unwrap());
    assert_eq!(mix.a().get(), 999);
    assert_eq!(mix.b().value(), 3);

    let mix = mix.with_b(u2::new(0));
    assert_eq!(mix.a().get(), 999);
    assert_eq!(mix.b().value(), 0);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 8)]
struct AnnotatedSizeLarger;

#[test]
fn annotated_size_larger() {
    let zst = AnnotatedSizeLarger.pack();
    assert_eq!(zst.into_raw(), 0);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32, nonzero)]
struct AbsolutePath {
    a: ::core::num::NonZeroU8,
    b: ribbit::u24,
}

#[test]
fn absolute_path() {
    let path = AbsolutePath {
        a: ::std::num::NonZeroU8::new(5).unwrap(),
        b: ribbit::u24::new(22),
    }
    .pack();
    assert_eq!(path.a().get(), 5);
    assert_eq!(path.b().value(), 22);
}

#[cfg(feature = "u128")]
#[expect(dead_code)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 128)]
struct Tuple128(u32, u32, u64);

#[cfg(feature = "u128")]
#[expect(dead_code)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 128, nonzero)]
struct NonZero128(ribbit::NonZeroU128);

#[cfg(feature = "u128")]
#[expect(dead_code)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 99)]
struct Arbitrary128(ribbit::u99);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 40)]
struct NativeSigned {
    a: i32,
    b: i8,
}

#[test]
fn native_signed() {
    let h = NativeSigned {
        a: 0xead_beef,
        b: 0xd,
    }
    .pack();

    assert_eq!(h.a(), 0xead_beef);
    assert_eq!(h.b(), 0xd);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct ArbitrarySigned {
    a: i40,
    b: i24,
}

#[test]
fn arbitrary_signed() {
    let h = ArbitrarySigned {
        a: i40::new(0xd_dead_beef),
        b: i24::new(0xe_efde),
    }
    .pack();

    assert_eq!(h.a().value(), 0xd_dead_beef);
    assert_eq!(h.b().value(), 0xe_efde);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct NonZeroSigned {
    a: NonZeroI32,
    b: NonZeroI8,
}

#[test]
fn nonzero_signed() {
    let h = NonZeroSigned {
        a: NonZeroI32::new(0xead_beef).unwrap(),
        b: NonZeroI8::new(0xd).unwrap(),
    }
    .pack();

    assert_eq!(h.a().get(), 0xead_beef);
    assert_eq!(h.b().get(), 0xd);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct RenameGet {
    #[ribbit(get(rename = "get_a"))]
    a: u32,
    #[ribbit(get(rename = "get_b"))]
    b: u32,
}

#[test]
fn rename_get() {
    let h = RenameGet {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.into_raw(), 0xbeef_dead_dead_beef);
    assert_eq!(h.get_a(), 0xdead_beef);
    assert_eq!(h.get_b(), 0xbeef_dead);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct SkipGet {
    #[ribbit(get(skip))]
    a: u32,
    #[ribbit(get(skip))]
    b: u32,
}

#[test]
fn skip_get() {
    let h = SkipGet {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.into_raw(), 0xbeef_dead_dead_beef);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct RenameWith {
    #[ribbit(with(rename = "update_a"))]
    a: u32,
    #[ribbit(with(rename = "update_b"))]
    b: u32,
}

#[test]
fn rename_with() {
    let h = RenameWith {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.into_raw(), 0xbeef_dead_dead_beef);
    assert_eq!(h.a(), 0xdead_beef);
    assert_eq!(h.b(), 0xbeef_dead);

    let h = h.update_a(0xbeef_dead).update_b(0xdead_beef);
    assert_eq!(h.into_raw(), 0xdead_beef_beef_dead);
    assert_eq!(h.a(), 0xbeef_dead);
    assert_eq!(h.b(), 0xdead_beef);
}

pub(crate) mod one {
    use ribbit::Pack;

    pub(crate) mod two {
        #![allow(clippy::too_many_arguments)]

        use ribbit::Pack;

        #[derive(ribbit::Pack, Copy, Clone, Default)]
        #[ribbit(size = 64)]
        pub struct Visibility {
            private_implicit: u8,
            pub(self) private_explicit: u8,
            #[rustfmt::skip]
            pub(in self) private_explicit_in: u8,

            pub(super) super_: u8,
            #[rustfmt::skip]
            pub(in super) super_in: u8,
            pub(in super::super) super_super: u8,

            pub(crate) crate_: u8,
            pub(in crate::one::two) crate_path: u8,
        }

        #[test]
        fn vis_private_ok() {
            let vis = Visibility::default().pack();
            assert_eq!(vis.private_implicit(), 0);
            assert_eq!(vis.private_explicit(), 0);
            assert_eq!(vis.private_explicit_in(), 0);
            assert_eq!(vis.crate_path(), 0);
        }
    }

    #[test]
    fn vis_super_ok() {
        let vis = crate::one::two::Visibility::default().pack();
        // assert_eq!(vis.private_implicit(), 0);
        // assert_eq!(vis.private_explicit(), 0);
        // assert_eq!(vis.private_explicit_in(), 0);
        // assert_eq!(vis.crate_path(), 0);
        assert_eq!(vis.super_(), 0);
        assert_eq!(vis.super_in(), 0);
    }
}

#[test]
fn vis_crate_ok() {
    use ribbit::Pack;
    let vis = crate::one::two::Visibility::default().pack();
    // assert_eq!(vis.private_implicit(), 0);
    // assert_eq!(vis.private_explicit(), 0);
    // assert_eq!(vis.private_explicit_in(), 0);
    // assert_eq!(vis.super_(), 0);
    // assert_eq!(vis.super_in(), 0);
    // assert_eq!(vis.crate_path(), 0);
    assert_eq!(vis.super_super(), 0);
    assert_eq!(vis.crate_(), 0);
}
