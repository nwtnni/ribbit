use core::marker::PhantomData;
use core::num::NonZeroU64;

use ribbit::Pack as _;
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 0)]
pub struct Foo;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
pub struct S {
    a: u64,
    #[expect(unused)]
    #[ribbit(size = 0)]
    foo: crate::Foo,
}

#[test]
fn custom_zst() {
    assert_eq!(core::mem::size_of::<ribbit::Packed<Foo>>(), 0);

    let h = S {
        a: 0xdead_beef,
        foo: Foo,
    }
    .pack();
    assert_eq!(h.into_raw(), 0xdead_beef);
    assert_eq!(h.a(), 0xdead_beef);
}

#[derive(ribbit::Pack)]
#[ribbit(size = 64)]
struct PhantomLast<A> {
    a: u64,
    #[ribbit(size = 0)]
    foo: PhantomData<A>,
}

impl<A> Copy for PhantomLast<A> {}

impl<A> Clone for PhantomLast<A> {
    fn clone(&self) -> Self {
        *self
    }
}

#[test]
fn phantom_last() {
    let h = PhantomLast::<usize> {
        a: 0xdead_beef,
        foo: PhantomData,
    }
    .pack();
    assert_eq!(h.into_raw(), 0xdead_beef);
    assert_eq!(h.a(), 0xdead_beef);
}

#[derive(ribbit::Pack)]
#[ribbit(size = 64, non_zero)]
struct Phantom<A> {
    a: NonZeroU64,
    #[ribbit(size = 0)]
    foo: PhantomData<A>,
}

impl<A> Copy for Phantom<A> {}

impl<A> Clone for Phantom<A> {
    fn clone(&self) -> Self {
        *self
    }
}

#[test]
fn phantom_nonzero() {
    let h = Phantom::<usize> {
        a: ribbit::NonZeroU64::new(0xdead_beef).unwrap(),
        foo: PhantomData,
    }
    .pack();
    assert_eq!(h.into_raw().get(), 0xdead_beef);
    assert_eq!(h.a().get(), 0xdead_beef);
}

#[derive(ribbit::Pack, Copy, Clone, Debug, PartialEq, Eq)]
#[ribbit(size = 0, derive(Debug, Eq))]
struct Zst;

#[test]
fn pack_zst() {
    assert_eq!(core::mem::size_of::<ribbit::Packed<Zst>>(), 0);

    let zst = Zst.pack();

    #[allow(clippy::let_unit_value)]
    let loose = ribbit::convert::packed_to_loose(zst);
    let packed = unsafe { ribbit::convert::loose_to_packed(loose) };

    assert_eq!(zst, packed);
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 32)]
struct LowOffset(#[ribbit(size = 0)] crate::Zst);

#[test]
fn low_offset() {
    let zst = Zst;
    let _hole = LowOffset(zst).pack();
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 32)]
struct HighOffset<T>(#[ribbit(offset = 32, size = 0)] T);

#[test]
fn high_offset() {
    let zst = Zst;
    let _hole = HighOffset(zst).pack();
}
