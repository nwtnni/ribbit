use core::marker::PhantomData;
use core::num::NonZeroU64;

use ribbit::Pack as _;
use ribbit::Unpack as _;

#[test]
fn custom_zst() {
    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 0)]
    struct Foo;

    assert_eq!(core::mem::size_of::<ribbit::Packed<Foo>>(), 0);

    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 64)]
    struct S {
        a: u64,
        #[expect(unused)]
        #[ribbit(size = 0)]
        foo: Foo,
    }

    let h = S {
        a: 0xdead_beef,
        foo: Foo,
    }
    .pack();
    assert_eq!(h.value, 0xdead_beef);
    assert_eq!(h.a(), 0xdead_beef);
}

#[test]
fn phantom() {
    #[ribbit::pack(size = 64)]
    struct Phantom<A> {
        a: u64,
        #[ribbit(size = 0)]
        foo: PhantomData<A>,
    }

    impl<A> Copy for Phantom<A> {}

    impl<A> Clone for Phantom<A> {
        fn clone(&self) -> Self {
            *self
        }
    }

    let h = Phantom::<usize> {
        a: 0xdead_beef,
        foo: PhantomData,
    }
    .pack();
    assert_eq!(h.value, 0xdead_beef);
    assert_eq!(h.a(), 0xdead_beef);
}

#[test]
fn phantom_nonzero() {
    #[ribbit::pack(size = 64, nonzero)]
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

    let h = Phantom::<usize> {
        a: NonZeroU64::new(0xdead_beef).unwrap(),
        foo: PhantomData,
    }
    .pack();
    assert_eq!(h.value.get(), 0xdead_beef);
    assert_eq!(h.a().get(), 0xdead_beef);
}

#[test]
fn pack_zst() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 0, debug, eq)]
    struct Foo;

    assert_eq!(core::mem::size_of::<ribbit::Packed<Foo>>(), 0);

    let zst = Foo.pack();

    #[allow(clippy::let_unit_value)]
    let loose = ribbit::convert::packed_to_loose(zst);
    let packed = unsafe { ribbit::convert::loose_to_packed(loose) };

    assert_eq!(zst, packed);
}

#[test]
fn pack_zst_large() {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 0)]
    struct Zst;

    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 32)]
    struct Hole(#[ribbit(size = 0)] Zst);

    let zst = Zst;
    let hole = Hole(zst).pack();
    assert_eq!(zst, hole._0().unpack());
}
