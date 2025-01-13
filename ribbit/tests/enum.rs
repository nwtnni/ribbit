#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
enum SingleNamed {
    #[ribbit(size = 16)]
    #[derive(Copy, Clone)]
    A { a: u16 },
}

#[test]
fn single_named() {
    let named = SingleNamed::new(SingleNamedUnpacked::A(A::new(5)));

    match named.unpack() {
        SingleNamedUnpacked::A(a) => assert_eq!(a.a(), 5),
    }
}

#[ribbit::pack(size = 8)]
#[derive(Copy, Clone)]
struct Byte(u8);

#[ribbit::pack(size = 8)]
#[derive(Copy, Clone)]
enum SingleNewtype {
    #[ribbit(size = 8)]
    Byte(Byte),
}

#[test]
fn single_newtype() {
    let b = SingleNewtype::new(SingleNewtypeUnpacked::Byte(Byte::new(3)));

    match b.unpack() {
        SingleNewtypeUnpacked::Byte(b) => assert_eq!(b._0(), 3),
    }
}

#[ribbit::pack(size = 8)]
#[derive(Copy, Clone)]
enum SingleUnit {
    Unit,
}

#[test]
fn single_unit() {
    let b = SingleUnit::new(SingleUnitUnpacked::Unit);

    match b.unpack() {
        SingleUnitUnpacked::Unit => (),
    }
}

#[ribbit::pack(size = 34)]
#[derive(Copy, Clone)]
enum Mixed {
    #[ribbit(size = 16)]
    #[derive(Copy, Clone)]
    X {
        a: u16,
    },
    Y(u32),
    Z,
}

#[test]
fn mixed() {
    let mut x = Mixed::new(MixedUnpacked::X(X::new(3)));

    match x.unpack() {
        MixedUnpacked::X(x) => assert_eq!(x.a(), 3),
        _ => unreachable!(),
    }

    x = Mixed::new(MixedUnpacked::Y(5));

    match x.unpack() {
        MixedUnpacked::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = Mixed::new(MixedUnpacked::Z);

    match x.unpack() {
        MixedUnpacked::Z => (),
        _ => unreachable!(),
    }
}

#[ribbit::pack(size = 8)]
#[derive(Copy, Clone)]
enum Wrapper {
    #[ribbit(size = 8)]
    Byte(u8),
}

#[test]
fn wrapper() {
    let b = Wrapper::new(WrapperUnpacked::Byte(3));

    match b.unpack() {
        WrapperUnpacked::Byte(b) => assert_eq!(b, 3),
    }
}
