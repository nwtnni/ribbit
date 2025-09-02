use ribbit::Pack as _;
use ribbit::Unpack as _;

#[test]
fn single_named() {
    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 16)]
    enum SingleNamed {
        #[ribbit(size = 16)]
        A { a: u16 },
    }

    let named = SingleNamed::A { a: 5 }.pack();

    match named.unpack() {
        SingleNamed::A { a } => assert_eq!(a, 5),
    }
}

#[test]
fn single_newtype() {
    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 8)]
    struct Byte(u8);

    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 8)]
    enum SingleNewtype {
        #[ribbit(size = 8)]
        Byte(Byte),
    }

    let b = SingleNewtype::Byte(Byte(3)).pack();

    match b.unpack() {
        SingleNewtype::Byte(b) => assert_eq!(b.0, 3),
    }
}

#[test]
fn single_unit() {
    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 8)]
    enum SingleUnit {
        #[ribbit(size = 0)]
        Unit,
    }

    let b = SingleUnit::Unit.pack();

    match b.unpack() {
        SingleUnit::Unit => (),
    }
}

#[test]
fn mixed() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 34, eq, debug)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 },
        #[ribbit(size = 32)]
        Y(u32),
        #[ribbit(size = 0)]
        Z,
    }

    let mut x = <ribbit::Pack![Mixed]>::new_x(3);
    assert_eq!(x, Mixed::X { a: 3 }.pack());
    match x.unpack() {
        Mixed::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = <ribbit::Pack![Mixed]>::new_y(5);
    assert_eq!(x, Mixed::Y(5).pack());
    match x.unpack() {
        Mixed::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = <ribbit::Pack![Mixed]>::new_z();
    assert_eq!(x, Mixed::Z.pack());
    match x.unpack() {
        Mixed::Z => (),
        _ => unreachable!(),
    }
}

#[test]
fn wrapper() {
    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 8)]
    enum Wrapper {
        #[ribbit(size = 8)]
        Byte(u8),
    }

    let b = Wrapper::Byte(3).pack();

    match b.unpack() {
        Wrapper::Byte(b) => assert_eq!(b, 3),
    }
}

#[test]
fn explicit_discriminant() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 48, eq, debug)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 } = 3,
        #[ribbit(size = 32)]
        Y(u32) = 16,
        #[ribbit(size = 0)]
        Z = 2,
    }

    let mut x = <ribbit::Pack![Mixed]>::new_x(3);
    assert_eq!(x, Mixed::X { a: 3 }.pack());
    match x.unpack() {
        Mixed::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = <ribbit::Pack![Mixed]>::new_y(5);
    assert_eq!(x, Mixed::Y(5).pack());
    match x.unpack() {
        Mixed::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = <ribbit::Pack![Mixed]>::new_z();
    assert_eq!(x, Mixed::Z.pack());
    match x.unpack() {
        Mixed::Z => (),
        _ => unreachable!(),
    }
}

#[test]
fn explicit_discriminant_nonzero() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 64, eq, debug, nonzero)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 } = 3,
        #[ribbit(size = 32)]
        Y(u32),
        #[ribbit(size = 0)]
        Z = 2,
    }

    assert_eq!(
        core::mem::size_of::<ribbit::Pack![Mixed]>(),
        core::mem::size_of::<ribbit::Pack![Option<Mixed>]>()
    );

    let mut x = Some(<ribbit::Pack![Mixed]>::new_x(3));
    assert_eq!(x, Some(Mixed::X { a: 3 }.pack()));
    match x.unpack() {
        Some(Mixed::X { a }) => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = Some(<ribbit::Pack![Mixed]>::new_y(5));
    assert_eq!(x, Some(Mixed::Y(5).pack()));
    match x.unpack() {
        Some(Mixed::Y(y)) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = Some(<ribbit::Pack![Mixed]>::new_z());
    assert_eq!(x, Some(Mixed::Z.pack()));
    match x.unpack() {
        Some(Mixed::Z) => (),
        _ => unreachable!(),
    }
}
