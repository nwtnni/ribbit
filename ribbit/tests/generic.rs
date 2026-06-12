use core::marker::PhantomData;

use ribbit::u3;
use ribbit::u7;
use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 48)]
struct Versioned<T> {
    version: u16,
    #[ribbit(size = 32)]
    inner: T,
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
struct A(u32);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
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

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 8, derive(Debug, Eq))]
enum EnumNewtype<T> {
    #[ribbit(size = 7)]
    Left(T),
    #[ribbit(size = 7)]
    Right(T),
}

#[test]
fn r#enum_newtype() {
    let a = EnumNewtype::Left(u7::new(1)).pack();
    let b = EnumNewtype::Right(u7::new(1)).pack();

    assert_ne!(a, b);

    match a.unpack() {
        EnumNewtype::Left(l) => assert_eq!(l.value(), 1),
        EnumNewtype::Right(_) => unreachable!(),
    }

    match b.unpack() {
        EnumNewtype::Left(_) => unreachable!(),
        EnumNewtype::Right(r) => assert_eq!(r.value(), 1),
    }
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 8, derive(Debug, Eq))]
enum EnumNamed<T> {
    #[ribbit(size = 7)]
    Left {
        #[ribbit(size = 7)]
        l: T,
    },
    #[ribbit(size = 7)]
    Right {
        #[ribbit(size = 7)]
        r: T,
    },
}

#[test]
fn r#enum_named() {
    let a = EnumNamed::Left { l: u7::new(1) }.pack();
    let b = EnumNamed::Right { r: u7::new(1) }.pack();

    assert_ne!(a, b);

    match a.unpack() {
        EnumNamed::Left { l } => assert_eq!(l.value(), 1),
        EnumNamed::Right { .. } => unreachable!(),
    }

    match b.unpack() {
        EnumNamed::Left { .. } => unreachable!(),
        EnumNamed::Right { r } => assert_eq!(r.value(), 1),
    }
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 3, derive(Debug, Eq))]
struct Small(ribbit::u3);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 24, derive(Debug))]
struct Large<T> {
    #[ribbit(size = 16)]
    a: T,
    b: u8,
}

#[test]
fn actual_size_lt_expected() {
    let a = Small(u3::new(3));
    let b = Large { a, b: 7 }.pack();

    assert_eq!(b.a(), a.pack());
    assert_eq!(b.b(), 7);
}

trait Foo {
    type Bar: Copy + core::fmt::Debug + Eq;
}

impl Foo for u32 {
    type Bar = u64;
}

#[derive(ribbit::Pack, Debug, PartialEq, Eq)]
#[ribbit(size = 64)]
struct Wrapper<A>(<A as crate::Foo>::Bar)
where
    A: crate::Foo;

impl<A> Clone for Wrapper<A>
where
    A: Foo,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for Wrapper<A> where A: Foo {}

#[test]
fn associated_type() {
    let wrapper = Wrapper::<u32>(5u64);
    assert_eq!(wrapper, wrapper.pack().unpack());
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ribbit::Pack)]
#[ribbit(size = 64)]
struct ConstNew<T> {
    other: u32,
    #[ribbit(size = 32)]
    data: T,
}

#[test]
fn const_new() {
    let outer = ConstNew::<u7> {
        data: u7::new(5),
        other: 10,
    };

    const {
        ribbit::Packed::<ConstNew<u7>>::new(10, u7::new(5));
    }

    assert_eq!(outer.pack().unpack(), outer);
    assert_eq!(outer.pack().data().value(), 5);
}

#[derive(Debug, PartialEq, Eq, ribbit::Pack)]
#[ribbit(size = 64, derive(Debug, Eq))]
struct Phantom<T> {
    data: u64,
    _type: ribbit::PhantomData<T>,
}

impl<T> Copy for Phantom<T> {}
impl<T> Clone for Phantom<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[test]
fn phantom_data_zst() {
    let unpacked = Phantom::<usize> {
        data: 34,
        _type: PhantomData,
    };

    let packed = ribbit::Packed::<Phantom<usize>>::new(34);

    assert_eq!(unpacked.pack(), packed);
    assert_eq!(unpacked, packed.unpack());
    assert_eq!(packed.data(), 34);
}
