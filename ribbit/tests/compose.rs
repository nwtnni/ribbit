use ribbit::Pack as _;
use ribbit::Unpack as _;

use arbitrary_int::u7;
use core::num::NonZeroU16;

#[test]
fn basic() {
    #[derive(Clone)]
    #[ribbit::pack(size = 32)]
    struct Low {
        a: u32,
    }

    #[derive(Clone)]
    #[ribbit::pack(size = 64)]
    struct Whole {
        #[ribbit(size = 32)]
        low: Low,
        b: u32,
    }
}

#[test]
fn nonzero() {
    #[derive(Clone)]
    #[ribbit::pack(size = 16, nonzero)]
    struct Low {
        a: NonZeroU16,
    }

    #[derive(Clone)]
    #[ribbit::pack(size = 48)]
    struct Whole {
        #[ribbit(size = 16, nonzero)]
        low: Low,
        b: u32,
    }
}

#[test]
fn option_nonzero() {
    #[derive(Clone)]
    #[ribbit::pack(size = 16, nonzero)]
    struct Low {
        a: NonZeroU16,
    }

    #[derive(Clone)]
    #[ribbit::pack(size = 48)]
    struct Whole {
        #[ribbit(size = 16)]
        low: Option<Low>,
        b: u32,
    }

    let whole = Whole { low: None, b: 3 }.pack();
    assert!(whole.low().is_none());
    let whole = whole.with_low(Some(
        Low {
            a: NonZeroU16::new(5).unwrap(),
        }
        .pack(),
    ));
    assert_eq!(whole.low().unwrap().a().get(), 5);
}

#[test]
fn relax() {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 7)]
    struct Small(u7);

    // Pack a smaller type into a larger hole
    #[derive(Clone)]
    #[ribbit::pack(size = 30)]
    struct Large(Small);

    let a = Small(u7::new(5));
    let b = Large(a).pack();
    assert_eq!(a, b._0().unpack());
}
