use arbitrary_int::*;
use core::num::NonZeroU128;
use core::num::NonZeroU16;

#[test]
fn basic() {
    #[ribbit::pack(size = 64)]
    struct Half {
        a: u32,
        b: u32,
    }

    let h = Half::new(0xdead_beef, 0xbeef_dead);
    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a(), 0xdead_beef);
    assert_eq!(h.b(), 0xbeef_dead);
}

#[test]
fn arbitrary_field() {
    #[ribbit::pack(size = 64)]
    struct Half {
        a: u40,
        b: u24,
    }

    let h = Half::new(u40::new(0xad_dead_beef), u24::new(0xbe_efde));

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a().value(), 0xad_dead_beef);
    assert_eq!(h.b().value(), 0xbe_efde);
}

#[test]
fn arbitrary_repr() {
    #[ribbit::pack(size = 9)]
    struct Half {
        a: u1,
        b: u8,
    }

    let h = Half::new(u1::new(1), 0b10101010);
    assert_eq!(h.a().value(), 0b1);
    assert_eq!(h.b(), 0b10101010);
}

#[test]
fn set_bit() {
    #[ribbit::pack(size = 2)]
    struct Bits {
        a: u1,
        b: u1,
    }

    let h = Bits::new(u1::new(0), u1::new(0));

    assert_eq!(h.a().value(), 0b0);
    assert_eq!(h.b().value(), 0b0);

    let i = h.with_a(true.into());

    assert_eq!(i.a().value(), 0b1);
    assert_eq!(i.b().value(), 0b0);

    let j = i.with_b(true.into());

    assert_eq!(j.a().value(), 0b1);
    assert_eq!(j.b().value(), 0b1);
}

#[test]
fn set_clobber() {
    #[ribbit::pack(size = 2)]
    struct Clobber {
        value: u2,
    }

    let c = Clobber::new(u2::new(0));

    assert_eq!(c.value(), u2::new(0b00));

    let c = c.with_value(u2::new(0b01));
    assert_eq!(c.value(), u2::new(0b01));

    let c = c.with_value(u2::new(0b10));
    assert_eq!(c.value(), u2::new(0b10));
}

#[test]
fn nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    struct NonZero {
        nonzero: NonZeroU16,
    }
}

#[test]
fn explicit() {
    #[ribbit::pack(size = 18)]
    struct Mix {
        #[ribbit(offset = 2)]
        a: NonZeroU16,
        b: u2,
    }

    let mix = Mix::new(NonZeroU16::new(55).unwrap(), u2::new(3));
    assert_eq!(mix.a().get(), 55);
    assert_eq!(mix.b().value(), 3);

    let mix = mix.with_a(NonZeroU16::new(999).unwrap());
    assert_eq!(mix.a().get(), 999);
    assert_eq!(mix.b().value(), 3);

    let mix = mix.with_b(u2::new(0));
    assert_eq!(mix.a().get(), 999);
    assert_eq!(mix.b().value(), 0);
}

#[test]
fn underflow() {
    #[ribbit::pack(size = 8)]
    struct Zst;

    let zst = Zst::new();
    assert_eq!(zst.value, 0);
}

#[test]
fn type_path() {
    #[ribbit::pack(size = 32, nonzero)]
    struct Path {
        a: ::std::num::NonZeroU8,
        b: ribbit::u24,
    }

    let path = Path::new(::std::num::NonZeroU8::new(5).unwrap(), ribbit::u24::new(22));
    assert_eq!(path.a().get(), 5);
    assert_eq!(path.b().value(), 22);
}

#[test]
fn u128() {
    #[ribbit::pack(size = 128)]
    struct Tuple(u32, u32, u64);

    #[ribbit::pack(size = 128, nonzero)]
    struct NonZero(NonZeroU128);

    #[ribbit::pack(size = 99)]
    struct Arbitrary(u99);
}
