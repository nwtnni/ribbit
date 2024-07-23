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
