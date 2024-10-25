use core::num::NonZeroU16;

#[test]
fn basic() {
    #[ribbit::pack(size = 32)]
    #[derive(Copy, Clone)]
    struct Low {
        a: u32,
    }

    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 64)]
    struct Whole {
        #[ribbit(size = 32)]
        low: Low,
        b: u32,
    }
}

#[test]
fn nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    #[derive(Copy, Clone)]
    struct Low {
        a: NonZeroU16,
    }

    #[ribbit::pack(size = 48)]
    #[derive(Copy, Clone)]
    struct Whole {
        #[ribbit(size = 16, nonzero)]
        low: Low,
        b: u32,
    }
}

#[test]
fn option_nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    #[derive(Copy, Clone)]
    struct Low {
        a: NonZeroU16,
    }

    #[ribbit::pack(size = 48)]
    #[derive(Copy, Clone)]
    struct Whole {
        #[ribbit(size = 16)]
        low: Option<Low>,
        b: u32,
    }

    let whole = Whole::new(None, 3);
    assert!(whole.low().is_none());
    let whole = whole.with_low(Some(Low::new(NonZeroU16::new(5).unwrap())));
    assert_eq!(whole.low().unwrap().a().get(), 5);
}
