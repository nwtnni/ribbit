use core::num::NonZeroU16;

use arbitrary_int::u2;
use arbitrary_int::u9;

#[test]
fn basic() {
    #[ribbit::pack(size = 64)]
    #[derive(Copy, Clone)]
    struct Half {
        a: u32,
        b: u32,
    }

    let h = Half {
        value: 0xbeef_dead_dead_beef,
    };

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a(), 0xdead_beef);
    assert_eq!(h.b(), 0xbeef_dead);
}

#[test]
fn arbitrary_field() {
    #[ribbit::pack(size = 64)]
    #[derive(Copy, Clone)]
    struct Half {
        a: u40,
        b: u24,
    }

    let h = Half {
        value: 0xbeef_dead_dead_beef,
    };

    assert_eq!(h.value, 0xbeef_dead_dead_beef);
    assert_eq!(h.a().value(), 0xad_dead_beef);
    assert_eq!(h.b().value(), 0xbe_efde);
}

#[test]
fn arbitrary_repr() {
    #[ribbit::pack(size = 9)]
    #[derive(Copy, Clone)]
    struct Half {
        a: u1,
        b: u8,
    }

    let h = Half {
        value: u9::new(0b101010101),
    };

    assert_eq!(h.a().value(), 0b1);
    assert_eq!(h.b(), 0b10101010);
}

#[test]
fn set_bit() {
    #[ribbit::pack(size = 2)]
    #[derive(Copy, Clone)]
    struct Bits {
        a: u1,
        b: u1,
    }

    let h = Bits {
        value: u2::new(0b00),
    };

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
    #[derive(Copy, Clone)]
    struct Clobber {
        value: u2,
    }

    let c = Clobber {
        value: u2::new(0b00),
    };

    assert_eq!(c.value(), u2::new(0b00));

    let c = c.with_value(u2::new(0b01));
    assert_eq!(c.value(), u2::new(0b01));

    let c = c.with_value(u2::new(0b10));
    assert_eq!(c.value(), u2::new(0b10));
}

#[test]
fn nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    #[derive(Copy, Clone)]
    struct NonZero {
        nonzero: NonZeroU16,
    }
}

#[test]
fn explicit() {
    #[ribbit::pack(size = 18)]
    #[derive(Copy, Clone)]
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
