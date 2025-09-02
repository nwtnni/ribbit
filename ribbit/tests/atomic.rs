use core::sync::atomic::Ordering;

use ribbit::u22;
use ribbit::u26;
use ribbit::u9;

#[test]
fn aligned() {
    use ribbit::atomic::Atomic32;

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 32, debug, eq)]
    struct Foo {
        lo: u16,
        hi: u16,
    }

    #[allow(clippy::disallowed_names)]
    let foo = Atomic32::<Foo>::new(Foo { lo: 5, hi: 10 });

    assert_eq!(
        foo.compare_exchange(
            Foo { lo: 5, hi: 10 },
            Foo { lo: 6, hi: 11 },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo { lo: 5, hi: 10 })
    );

    assert_eq!(foo.load(Ordering::Relaxed), Foo { lo: 6, hi: 11 });
}

#[test]
fn unaligned() {
    use ribbit::atomic::Atomic32;

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 32, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u22,
    }

    #[allow(clippy::disallowed_names)]
    let foo = Atomic32::<Foo>::new(Foo {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    assert_eq!(
        foo.compare_exchange(
            Foo {
                lo: 5u8.into(),
                hi: 10u8.into()
            },
            Foo {
                lo: 6u8.into(),
                hi: 11u8.into()
            },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo {
            lo: 5u8.into(),
            hi: 10u8.into()
        })
    );

    assert_eq!(
        foo.load(Ordering::Relaxed),
        Foo {
            lo: 6u8.into(),
            hi: 11u8.into()
        },
    );
}

#[test]
fn undersized() {
    use ribbit::atomic::Atomic64;

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 64, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u26,
    }

    #[allow(clippy::disallowed_names)]
    let foo = Atomic64::<Foo>::new(Foo {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    assert_eq!(
        foo.compare_exchange(
            Foo {
                lo: 5u8.into(),
                hi: 10u8.into()
            },
            Foo {
                lo: 6u8.into(),
                hi: 11u8.into()
            },
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo {
            lo: 5u8.into(),
            hi: 10u8.into()
        })
    );

    assert_eq!(
        foo.load(Ordering::Relaxed),
        Foo {
            lo: 6u8.into(),
            hi: 11u8.into()
        }
    );
}

#[test]
fn unique() {
    use ribbit::atomic::Atomic64;

    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 64, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u26,
    }

    #[allow(clippy::disallowed_names)]
    let mut foo = Atomic64::<Foo>::new(Foo {
        lo: 5u8.into(),
        hi: 10u8.into(),
    });

    foo.set(Foo {
        lo: 9u8.into(),
        hi: 3u8.into(),
    });

    assert_eq!(
        foo.get(),
        Foo {
            lo: 9u8.into(),
            hi: 3u8.into()
        },
    );
}
