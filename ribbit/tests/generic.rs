use arbitrary_int::u3;
use arbitrary_int::u7;

#[ribbit::pack(size = 48)]
struct Versioned<T> {
    version: u16,
    #[ribbit(size = 32)]
    inner: T,
}

#[ribbit::pack(size = 32)]
struct A(u32);

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

// #[test]
// fn r#enum_newtype() {
//     #[ribbit::pack(size = 8, debug, eq)]
//     enum Either<T> {
//         #[ribbit(size = 7)]
//         Left(T),
//         #[ribbit(size = 7)]
//         Right(T),
//     }
//
//     let a = Either::new(EitherUnpacked::Left(u7::new(1)));
//     let b = Either::new(EitherUnpacked::Right(u7::new(1)));
//
//     assert_ne!(a, b);
//
//     match a.unpack() {
//         EitherUnpacked::Left(l) => assert_eq!(l.value(), 1),
//         EitherUnpacked::Right(_) => unreachable!(),
//     }
//
//     match b.unpack() {
//         EitherUnpacked::Left(_) => unreachable!(),
//         EitherUnpacked::Right(r) => assert_eq!(r.value(), 1),
//     }
// }
//
// #[test]
// fn r#enum_named() {
//     #[ribbit::pack(size = 8, debug, eq)]
//     enum Either<T> {
//         #[ribbit(size = 7, debug, from)]
//         Left {
//             #[ribbit(size = 7)]
//             l: T,
//         },
//         #[ribbit(size = 7, debug, from)]
//         Right {
//             #[ribbit(size = 7)]
//             r: T,
//         },
//     }
//
//     let a = Either::new(EitherUnpacked::Left(Left::new(u7::new(1))));
//     let b = Either::new(EitherUnpacked::Right(Right::new(u7::new(1))));
//
//     assert_eq!(a, Left::new(u7::new(1)).into());
//     assert_eq!(b, Right::new(u7::new(1)).into());
//
//     match a.unpack() {
//         EitherUnpacked::Left(l) => assert_eq!(l.l().value(), 1),
//         EitherUnpacked::Right(_) => unreachable!(),
//     }
//
//     match b.unpack() {
//         EitherUnpacked::Left(_) => unreachable!(),
//         EitherUnpacked::Right(r) => assert_eq!(r.r().value(), 1),
//     }
// }

#[test]
fn relax() {
    #[ribbit::pack(size = 3, debug, eq)]
    struct Small(u3);

    #[ribbit::pack(size = 24, debug)]
    struct Large<T> {
        #[ribbit(size = 16)]
        a: T,
        b: u8,
    }

    let a = Small(u3::new(3));
    let b = Large { a, b: 7 }.pack();

    assert_eq!(b.a().unpack(), a);
    assert_eq!(b.b().unpack(), 7);
}
