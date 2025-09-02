use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(Copy, Clone)]
#[ribbit::pack(size = 16)]
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
}

#[derive(Copy, Clone)]
#[ribbit::pack(size = 8)]
struct Byte(u8);

#[derive(Copy, Clone)]
#[ribbit::pack(size = 8)]
enum SingleNewtype {
    #[ribbit(size = 8)]
    Byte(Byte),
}

#[test]
fn single_newtype() {
    let b = SingleNewtype::Byte(Byte(3)).pack();

    match b.unpack() {
        SingleNewtype::Byte(b) => assert_eq!(b.0, 3),
    }
}

#[derive(Copy, Clone)]
#[ribbit::pack(size = 8)]
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

#[test]
fn mixed() {
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

#[derive(Copy, Clone)]
#[ribbit::pack(size = 8)]
enum Wrapper {
    #[ribbit(size = 8)]
    Byte(u8),
}

#[test]
fn wrapper() {
    let b = Wrapper::Byte(3).pack();

    match b.unpack() {
        Wrapper::Byte(b) => assert_eq!(b, 3),
    }
}
