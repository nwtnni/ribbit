use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 16, debug, eq)]
enum SingleNamed {
    #[ribbit(size = 16)]
    A { a: u16 },
}

#[test]
fn single_named() {
    let named = SingleNamed::A { a: 5 }.pack();

    match named.unpack() {
        SingleNamed::A { a } => assert_eq!(a, 5),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<SingleNamed>::new_unchecked(named.into_raw()) },
        named
    );
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 8)]
struct Byte(u8);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 8)]
enum SingleNewtype {
    #[ribbit(size = 8)]
    Byte(crate::Byte),
}

#[test]
fn single_newtype() {
    let b = SingleNewtype::Byte(Byte(3)).pack();

    match b.unpack() {
        SingleNewtype::Byte(b) => assert_eq!(b.0, 3),
    }
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 8)]
enum SingleUnit {
    #[ribbit(size = 0)]
    Unit,
}

#[test]
fn single_unit() {
    let b = SingleUnit::Unit.pack();

    match b.unpack() {
        SingleUnit::Unit => (),
    }
}

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

#[test]
fn mixed() {
    let mut x = ribbit::Packed::<Mixed>::new_x(3);
    assert_eq!(x, Mixed::X { a: 3 }.pack());
    match x.unpack() {
        Mixed::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.into_raw()) }.unpack(),
        Mixed::X { a: 3 },
    );

    x = ribbit::Packed::<Mixed>::new_y(5);
    assert_eq!(x, Mixed::Y(5).pack());
    match x.unpack() {
        Mixed::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.into_raw()) }.unpack(),
        Mixed::Y(5),
    );

    x = ribbit::Packed::<Mixed>::new_z();
    assert_eq!(x, Mixed::Z.pack());
    match x.unpack() {
        Mixed::Z => (),
        _ => unreachable!(),
    }

    assert_eq!(
        unsafe { ribbit::Packed::<Mixed>::new_unchecked(x.into_raw()) }.unpack(),
        Mixed::Z,
    );
}

#[repr(u8)]
#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 48, eq, debug)]
enum Discriminant {
    #[ribbit(size = 16)]
    X { a: u16 } = 3,
    #[ribbit(size = 32)]
    Y(u32) = 16,
    #[ribbit(size = 0)]
    Z = 2,
}

#[test]
fn discriminant() {
    let mut x = ribbit::Packed::<Discriminant>::new_x(3);
    assert_eq!(x, Discriminant::X { a: 3 }.pack());
    match x.unpack() {
        Discriminant::X { a } => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = ribbit::Packed::<Discriminant>::new_y(5);
    assert_eq!(x, Discriminant::Y(5).pack());
    match x.unpack() {
        Discriminant::Y(y) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = ribbit::Packed::<Discriminant>::new_z();
    assert_eq!(x, Discriminant::Z.pack());
    match x.unpack() {
        Discriminant::Z => (),
        _ => unreachable!(),
    }
}

#[repr(u8)]
#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 64, eq, debug, nonzero)]
enum NonZero {
    #[ribbit(size = 16)]
    X { a: u16 } = 3,
    #[ribbit(size = 32)]
    Y(u32),
    #[ribbit(size = 0)]
    Z = 2,
}

#[test]
fn nonzero() {
    assert_eq!(
        core::mem::size_of::<ribbit::Packed::<NonZero>>(),
        core::mem::size_of::<ribbit::Packed::<Option<NonZero>>>()
    );

    let mut x = Some(ribbit::Packed::<NonZero>::new_x(3));
    assert_eq!(x, Some(NonZero::X { a: 3 }.pack()));
    match x.unpack() {
        Some(NonZero::X { a }) => assert_eq!(a, 3),
        _ => unreachable!(),
    }

    x = Some(ribbit::Packed::<NonZero>::new_y(5));
    assert_eq!(x, Some(NonZero::Y(5).pack()));
    match x.unpack() {
        Some(NonZero::Y(y)) => assert_eq!(y, 5),
        _ => unreachable!(),
    }

    x = Some(ribbit::Packed::<NonZero>::new_z());
    assert_eq!(x, Some(NonZero::Z.pack()));
    match x.unpack() {
        Some(NonZero::Z) => (),
        _ => unreachable!(),
    }
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 8, debug, eq)]
enum UnitOmitSize {
    A,
    B,
    C,
}

#[test]
fn unit_omit_size() {
    let a = UnitOmitSize::A.pack();
    let c = UnitOmitSize::C.pack();
    assert_ne!(a, c);
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 8, debug, eq)]
enum UnitDiscriminant {
    A = 1,
    B = 5,
    C = 3,
}

#[test]
fn unit_discriminant() {
    let a = UnitDiscriminant::A.pack();
    let c = UnitDiscriminant::C.pack();
    assert_ne!(a, c);

    assert_eq!(
        UnitDiscriminant::A.pack().into_raw(),
        UnitDiscriminant::A as u8
    );
    assert_eq!(
        UnitDiscriminant::B.pack().into_raw(),
        UnitDiscriminant::B as u8
    );
    assert_eq!(
        UnitDiscriminant::C.pack().into_raw(),
        UnitDiscriminant::C as u8
    );
}
