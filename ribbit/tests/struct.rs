use arbitrary_int::*;
use core::num::NonZeroI32;
use core::num::NonZeroI8;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use ribbit::Pack as _;

#[test]
fn basic() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 64)]
    struct Half {
        a: u32,
        b: u32,
    }

    let h = Half {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a(), 0xdead_beef);
    assert_eq!(h.b(), 0xbeef_dead);
}

#[test]
fn arbitrary_field() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 64)]
    struct Half {
        a: u40,
        b: u24,
    }

    let h = Half {
        a: u40::new(0xad_dead_beef),
        b: u24::new(0xbe_efde),
    }
    .pack();

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a().value(), 0xad_dead_beef);
    assert_eq!(h.b().value(), 0xbe_efde);
}

#[test]
fn arbitrary_repr() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 9)]
    struct Half {
        a: u1,
        b: u8,
    }

    let h = Half {
        a: u1::new(1),
        b: 0b10101010,
    }
    .pack();
    assert_eq!(h.a().value(), 0b1);
    assert_eq!(h.b(), 0b10101010);
}

#[test]
fn set_bit() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 2)]
    struct Bits {
        a: u1,
        b: u1,
    }

    let h = Bits {
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

#[test]
fn set_clobber() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 2)]
    struct Clobber {
        value: u2,
    }

    let c = Clobber { value: u2::new(0) }.pack();

    assert_eq!(c.value(), u2::new(0b00));

    let c = c.with_value(u2::new(0b01));
    assert_eq!(c.value(), u2::new(0b01));

    let c = c.with_value(u2::new(0b10));
    assert_eq!(c.value(), u2::new(0b10));
}

#[test]
fn nonzero() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 16, nonzero)]
    struct NonZero {
        nonzero: NonZeroU16,
    }
}

#[test]
fn explicit() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 18)]
    struct Mix {
        #[ribbit(offset = 2)]
        a: NonZeroU16,
        b: u2,
    }

    let mix = Mix {
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

#[test]
fn underflow() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 8)]
    struct Zst;

    let zst = Zst.pack();
    assert_eq!(zst.value, 0);
}

#[test]
fn type_path() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 32, nonzero)]
    struct Path {
        a: ::std::num::NonZeroU8,
        b: ribbit::u24,
    }

    let path = Path {
        a: ::std::num::NonZeroU8::new(5).unwrap(),
        b: ribbit::u24::new(22),
    }
    .pack();
    assert_eq!(path.a().get(), 5);
    assert_eq!(path.b().value(), 22);
}

#[test]
fn u128() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 128)]
    struct Tuple(u32, u32, u64);

    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 128, nonzero)]
    struct NonZero(NonZeroU128);

    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 99)]
    struct Arbitrary(u99);
}

#[test]
fn basic_signed() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 40)]
    struct Half {
        a: i32,
        b: i8,
    }

    let h = Half {
        a: 0xead_beef,
        b: 0xd,
    }
    .pack();

    assert_eq!(h.a(), 0xead_beef);
    assert_eq!(h.b(), 0xd);
}

#[test]
fn arbitrary_signed() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 64)]
    struct Half {
        a: i40,
        b: i24,
    }

    let h = Half {
        a: i40::new(0xd_dead_beef),
        b: i24::new(0xe_efde),
    }
    .pack();

    assert_eq!(h.a().value(), 0xd_dead_beef);
    assert_eq!(h.b().value(), 0xe_efde);
}

#[test]
fn nonzero_signed() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 64)]
    struct Half {
        a: NonZeroI32,
        b: NonZeroI8,
    }

    let h = Half {
        a: NonZeroI32::new(0xead_beef).unwrap(),
        b: NonZeroI8::new(0xd).unwrap(),
    }
    .pack();

    assert_eq!(h.a().get(), 0xead_beef);
    assert_eq!(h.b().get(), 0xd);
}

#[test]
fn rename_get() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 64)]
    struct Half {
        #[ribbit(get(rename = "b"))]
        a: u32,
        #[ribbit(get(rename = "a"))]
        b: u32,
    }

    let h = Half {
        a: 0xdead_beef,
        b: 0xbeef_dead,
    }
    .pack();

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.b(), 0xdead_beef);
    assert_eq!(h.a(), 0xbeef_dead);
}
