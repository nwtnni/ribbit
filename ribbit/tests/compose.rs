use arbitrary_int::u7;
use core::num::NonZeroU16;

#[test]
fn basic() {
    #[ribbit::pack(size = 32)]
    struct Low {
        a: u32,
    }

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
    struct Low {
        a: NonZeroU16,
    }

    #[ribbit::pack(size = 48)]
    struct Whole {
        #[ribbit(size = 16, nonzero)]
        low: Low,
        b: u32,
    }
}

#[test]
fn option_nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    struct Low {
        a: NonZeroU16,
    }

    #[ribbit::pack(size = 48)]
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

#[test]
fn relax() {
    #[ribbit::pack(size = 7, debug, eq)]
    struct Small(u7);

    // Pack a smaller type into a larger hole
    #[ribbit::pack(size = 30)]
    struct Large(Small);

    let a = Small::new(u7::new(5));
    let b = Large::new(a);
    assert_eq!(a, b._0());
}
