#![cfg(feature = "atomic")]

use core::sync::atomic::Ordering;

use ribbit::u22;
use ribbit::u26;
use ribbit::u9;
use ribbit::Atomic;

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 32, derive(Debug, Eq))]
struct Aligned {
    lo: u16,
    hi: u16,
}

#[test]
fn aligned() {
    let aligned = Atomic::<Aligned>::new(Aligned { lo: 5, hi: 10 });

    assert_eq!(
        aligned.compare_exchange(
            Aligned { lo: 5, hi: 10 },
            Aligned { lo: 6, hi: 11 },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Aligned { lo: 5, hi: 10 })
    );

    assert_eq!(aligned.load(Ordering::Relaxed), Aligned { lo: 6, hi: 11 });
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 32, derive(Debug, Eq))]
struct Unaligned {
    lo: u9,
    hi: u22,
}

#[test]
fn unaligned() {
    #[allow(clippy::disallowed_names)]
    let unaligned = Atomic::<Unaligned>::new(Unaligned {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    assert_eq!(
        unaligned.compare_exchange(
            Unaligned {
                lo: 5u8.into(),
                hi: 10u8.into()
            },
            Unaligned {
                lo: 6u8.into(),
                hi: 11u8.into()
            },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Unaligned {
            lo: 5u8.into(),
            hi: 10u8.into()
        })
    );

    assert_eq!(
        unaligned.load(Ordering::Relaxed),
        Unaligned {
            lo: 6u8.into(),
            hi: 11u8.into()
        },
    );
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 64, derive(Debug, Eq))]
struct Undersized {
    lo: u9,
    hi: u26,
}

#[test]
fn undersized() {
    #[allow(clippy::disallowed_names)]
    let undersized = Atomic::<Undersized>::new(Undersized {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    assert_eq!(
        undersized.compare_exchange(
            Undersized {
                lo: 5u8.into(),
                hi: 10u8.into()
            },
            Undersized {
                lo: 6u8.into(),
                hi: 11u8.into()
            },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Undersized {
            lo: 5u8.into(),
            hi: 10u8.into()
        })
    );

    assert_eq!(
        undersized.load(Ordering::Relaxed),
        Undersized {
            lo: 6u8.into(),
            hi: 11u8.into()
        }
    );
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 64, derive(Debug, Eq))]
struct Mutable {
    lo: u9,
    hi: u26,
}

#[test]
fn mutable() {
    #[allow(clippy::disallowed_names)]
    let mut mutable = Atomic::<Mutable>::new(Mutable {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    mutable.set(Mutable {
        lo: 9u8.into(),
        hi: 3u8.into(),
    });

    assert_eq!(
        mutable.get(),
        Mutable {
            lo: 9u8.into(),
            hi: 3u8.into()
        },
    );
}
