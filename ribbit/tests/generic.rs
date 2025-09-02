use ribbit::u3;
use ribbit::u7;
use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(Clone)]
#[ribbit::pack(size = 48)]
struct Versioned<T> {
    version: u16,
    #[ribbit(size = 32)]
    inner: T,
}

#[derive(Clone)]
#[ribbit::pack(size = 32)]
struct A(u32);

#[derive(Clone)]
#[ribbit::pack(size = 32)]
struct B {
    hi: u16,
    lo: u16,
}

#[test]
fn basic() {
    let a = Versioned {
        version: 15,
        inner: A(36),
    }
    .pack();

    assert_eq!(a.version(), 15);
    assert_eq!(a.inner()._0(), 36);

    let a = a.with_version(99);

    assert_eq!(a.version(), 99);
    assert_eq!(a.inner()._0(), 36);

    let a = a.with_inner(A(52).pack());

    assert_eq!(a.version(), 99);
    assert_eq!(a.inner()._0(), 52);
}

#[test]
fn compose() {
    let b = Versioned {
        version: 15,
        inner: B { hi: 1, lo: 2 },
    }
    .pack();

    assert_eq!(b.version(), 15);
    assert_eq!(b.inner().hi(), 1);
    assert_eq!(b.inner().lo(), 2);

    let b = b.with_version(99);

    assert_eq!(b.version(), 99);
    assert_eq!(b.inner().hi(), 1);
    assert_eq!(b.inner().lo(), 2);

    let b = b.with_inner(b.inner().with_hi(5));

    assert_eq!(b.version(), 99);
    assert_eq!(b.inner().hi(), 5);
    assert_eq!(b.inner().lo(), 2);
}

#[test]
fn r#enum_newtype() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 8, debug, eq)]
    enum Either<T> {
        #[ribbit(size = 7)]
        Left(T),
        #[ribbit(size = 7)]
        Right(T),
    }

    let a = Either::Left(u7::new(1)).pack();
    let b = Either::Right(u7::new(1)).pack();

    assert_ne!(a, b);

    match a.unpack() {
        Either::Left(l) => assert_eq!(l.value(), 1),
        Either::Right(_) => unreachable!(),
    }

    match b.unpack() {
        Either::Left(_) => unreachable!(),
        Either::Right(r) => assert_eq!(r.value(), 1),
    }
}

#[test]
fn r#enum_named() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 8, debug, eq)]
    enum Either<T> {
        #[ribbit(size = 7, debug, from)]
        Left {
            #[ribbit(size = 7)]
            l: T,
        },
        #[ribbit(size = 7, debug, from)]
        Right {
            #[ribbit(size = 7)]
            r: T,
        },
    }

    let a = Either::Left { l: u7::new(1) }.pack();
    let b = Either::Right { r: u7::new(1) }.pack();

    assert_ne!(a, b);

    match a.unpack() {
        Either::Left { l } => assert_eq!(l.value(), 1),
        Either::Right { .. } => unreachable!(),
    }

    match b.unpack() {
        Either::Left { .. } => unreachable!(),
        Either::Right { r } => assert_eq!(r.value(), 1),
    }
}

#[test]
fn relax() {
    #[derive(Clone, Debug)]
    #[ribbit::pack(size = 3, debug, eq)]
    struct Small(u3);

    #[derive(Clone)]
    #[ribbit::pack(size = 24, debug)]
    struct Large<T> {
        #[ribbit(size = 16)]
        a: T,
        b: u8,
    }

    let a = Small(u3::new(3));
    let b = Large { a: a.clone(), b: 7 }.pack();

    assert_eq!(b.a(), a.pack());
    assert_eq!(b.b(), 7);
}

#[test]
fn associated() {
    trait Foo {
        type Bar: Copy;
    }

    impl Foo for u32 {
        type Bar = u64;
    }

    #[ribbit::pack(size = 64)]
    struct Wrapper<A>(<A as Foo>::Bar)
    where
        A: Foo;

    impl<A> Clone for Wrapper<A>
    where
        A: Foo,
    {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<A> Copy for Wrapper<A> where A: Foo {}
}
