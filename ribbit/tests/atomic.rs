use core::sync::atomic::Ordering;

use ribbit::u22;
use ribbit::u26;
use ribbit::u9;

#[test]
fn aligned() {
    use ribbit::atomic::A32;

    #[ribbit::pack(size = 32, debug, eq)]
    struct Foo {
        lo: u16,
        hi: u16,
    }

    #[allow(clippy::disallowed_names)]
    let foo = A32::new(Foo::new(5, 10));

    assert_eq!(
        foo.compare_exchange(
            Foo::new(5, 10),
            Foo::new(6, 11),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo::new(5, 10))
    );

    assert_eq!(foo.load(Ordering::Relaxed), Foo::new(6, 11));
}

#[test]
fn unaligned() {
    use ribbit::atomic::A32;

    #[ribbit::pack(size = 32, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u22,
    }

    #[allow(clippy::disallowed_names)]
    let foo = A32::new(Foo::new(5u8.into(), 10u8.into()));

    assert_eq!(
        foo.compare_exchange(
            Foo::new(5u8.into(), 10u8.into()),
            Foo::new(6u8.into(), 11u8.into()),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo::new(5u8.into(), 10u8.into()))
    );

    assert_eq!(
        foo.load(Ordering::Relaxed),
        Foo::new(6u8.into(), 11u8.into())
    );
}

#[test]
fn undersized() {
    use ribbit::atomic::A64;

    #[ribbit::pack(size = 64, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u26,
    }

    #[allow(clippy::disallowed_names)]
    let foo = A64::new(Foo::new(5u8.into(), 10u8.into()));

    assert_eq!(
        foo.compare_exchange(
            Foo::new(5u8.into(), 10u8.into()),
            Foo::new(6u8.into(), 11u8.into()),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        Ok(Foo::new(5u8.into(), 10u8.into()))
    );

    assert_eq!(
        foo.load(Ordering::Relaxed),
        Foo::new(6u8.into(), 11u8.into())
    );
}

#[test]
fn unique() {
    use ribbit::atomic::A64;

    #[ribbit::pack(size = 64, debug, eq)]
    struct Foo {
        lo: u9,
        hi: u26,
    }

    #[allow(clippy::disallowed_names)]
    let mut foo = A64::new(Foo::new(5u8.into(), 10u8.into()));

    foo.set(Foo::new(9u8.into(), 3u8.into()));

    assert_eq!(foo.get(), Foo::new(9u8.into(), 3u8.into()),);
}
