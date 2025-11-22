use ribbit::Pack as _;
use ribbit::Unpack as _;

#[test]
fn single_named() {
    #[derive(ribbit::Pack, Copy, Clone, Debug)]
    #[ribbit(size = 16, debug, eq)]
    enum SingleNamed {
        #[ribbit(size = 16)]
        A { a: u16 },
    }

    let named = SingleNamed::A { a: 5 }.pack();

    match named.unpack() {
        SingleNamed::A { a } => assert_eq!(a, 5),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<SingleNamed>::new_unchecked(named.value) },
        named
    );
}

#[test]
fn single_newtype() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 8)]
    struct Byte(u8);

    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 8)]
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
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 8)]
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
    #[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
    #[ribbit(size = 34, eq, debug)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 },
        #[ribbit(size = 32)]
        Y(u32),
        #[ribbit(size = 0)]
        Z,
    }

    let mut x = ribbit::Packed::<Mixed>::new_x(3);
    assert_eq!(x, Mixed::X { a: 3 }.pack());
    match x.unpack() {
        Mixed::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.value) }.unpack(),
        Mixed::X { a: 3 },
    );

    x = ribbit::Packed::<Mixed>::new_y(5);
    assert_eq!(x, Mixed::Y(5).pack());
    match x.unpack() {
        Mixed::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.value) }.unpack(),
        Mixed::Y(5),
    );

    x = ribbit::Packed::<Mixed>::new_z();
    assert_eq!(x, Mixed::Z.pack());
    match x.unpack() {
        Mixed::Z => (),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.value) }.unpack(),
        Mixed::Z,
    );
}

#[test]
fn wrapper() {
    #[derive(ribbit::Pack, Copy, Clone)]
    #[ribbit(size = 8)]
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
    #[repr(u8)]
    #[derive(ribbit::Pack, Copy, Clone, Debug)]
    #[ribbit(size = 48, eq, debug)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 } = 3,
        #[ribbit(size = 32)]
        Y(u32) = 16,
        #[ribbit(size = 0)]
        Z = 2,
    }

    let mut x = ribbit::Packed::<Mixed>::new_x(3);
    assert_eq!(x, Mixed::X { a: 3 }.pack());
    match x.unpack() {
        Mixed::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = ribbit::Packed::<Mixed>::new_y(5);
    assert_eq!(x, Mixed::Y(5).pack());
    match x.unpack() {
        Mixed::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = ribbit::Packed::<Mixed>::new_z();
    assert_eq!(x, Mixed::Z.pack());
    match x.unpack() {
        Mixed::Z => (),
        _ => unreachable!(),
    }
}

#[test]
fn explicit_discriminant_nonzero() {
    #[repr(u8)]
    #[derive(ribbit::Pack, Copy, Clone, Debug)]
    #[ribbit(size = 64, eq, debug, nonzero)]
    enum Mixed {
        #[ribbit(size = 16)]
        X { a: u16 } = 3,
        #[ribbit(size = 32)]
        Y(u32),
        #[ribbit(size = 0)]
        Z = 2,
    }

    assert_eq!(
        core::mem::size_of::<ribbit::Packed::<Mixed>>(),
        core::mem::size_of::<ribbit::Packed::<Option<Mixed>>>()
    );

    let mut x = Some(ribbit::Packed::<Mixed>::new_x(3));
    assert_eq!(x, Some(Mixed::X { a: 3 }.pack()));
    match x.unpack() {
        Some(Mixed::X { a }) => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = Some(ribbit::Packed::<Mixed>::new_y(5));
    assert_eq!(x, Some(Mixed::Y(5).pack()));
    match x.unpack() {
        Some(Mixed::Y(y)) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = Some(ribbit::Packed::<Mixed>::new_z());
    assert_eq!(x, Some(Mixed::Z.pack()));
    match x.unpack() {
        Some(Mixed::Z) => (),
        _ => unreachable!(),
    }
}

#[test]
fn unit_omit_size() {
    #[derive(ribbit::Pack, Copy, Clone, Debug)]
    #[ribbit(size = 8, debug, eq)]
    enum Unit {
        A,
        B,
        C,
    }

    let a = Unit::A.pack();
    let c = Unit::C.pack();
    assert_ne!(a, c);
}
