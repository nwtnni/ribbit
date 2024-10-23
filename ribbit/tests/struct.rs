use arbitrary_int::u2;
use arbitrary_int::u9;

#[test]
fn basic() {
    #[ribbit::pack(size = 64)]
    #[derive(Debug)]
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
    #[derive(Debug)]
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
    #[derive(Debug)]
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
    #[derive(Debug)]
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
    #[derive(Debug)]
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
    struct NonZero {
        #[ribbit(nonzero = true)]
        nonzero: NonZeroU16,
    }
}
