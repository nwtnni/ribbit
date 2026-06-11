use ribbit::Pack as _;
use ribbit::Unpack as _;

use arbitrary_int::u7;
use core::num::NonZeroU16;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
struct Low {
    a: u32,
}

#[expect(dead_code)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct Whole {
    #[ribbit(size = 32)]
    low: crate::Low,
    b: u32,
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16, nonzero)]
struct LowNonZero {
    a: ribbit::NonZeroU16,
}

#[expect(dead_code)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 48)]
struct WholeNonZero {
    #[ribbit(size = 16, nonzero)]
    low: crate::LowNonZero,
    b: u32,
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 48)]
struct WholeNonZeroOption {
    #[ribbit(size = 16)]
    low: Option<crate::LowNonZero>,
    b: u32,
}

#[test]
fn option_nonzero() {
    let whole = WholeNonZeroOption { low: None, b: 3 }.pack();
    assert!(whole.low().is_none());
    let whole = whole.with_low(Some(
        LowNonZero {
            a: NonZeroU16::new(5).unwrap(),
        }
        .pack(),
    ));
    assert_eq!(whole.low().unwrap().a().get(), 5);
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 7)]
struct Small(ribbit::u7);

// Pack a smaller type into a larger hole
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 30)]
struct Large(#[ribbit(size = 7)] crate::Small);

#[test]
fn actual_size_lt_annotated() {
    let a = Small(u7::new(5));
    let b = Large(a).pack();
    assert_eq!(a, b._0().unpack());
}
