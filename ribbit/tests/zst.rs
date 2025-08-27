use core::marker::PhantomData;
use core::num::NonZeroU64;

use ribbit::Pack as _;
use ribbit::Unpack as _;

#[test]
fn custom_zst() {
    #[derive(Clone)]
    #[ribbit::pack(size = 0)]
    struct Foo;

    #[derive(Clone)]
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

    impl<A> Clone for Phantom<A> {
        fn clone(&self) -> Self {
            Self {
                a: self.a,
                foo: self.foo,
            }
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

    impl<A> Clone for Phantom<A> {
        fn clone(&self) -> Self {
            Self {
                a: self.a,
                foo: self.foo,
            }
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
    #[derive(Clone)]
    #[ribbit::pack(size = 0, debug, eq)]
    struct Foo;

    let zst = Foo.pack();

    #[allow(clippy::let_unit_value)]
    let loose = ribbit::convert::packed_to_loose::<Foo>(zst);
    let packed = unsafe { ribbit::convert::loose_to_packed::<Foo>(loose) };

    assert_eq!(zst, packed);
}

#[test]
fn pack_zst_large() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[ribbit::pack(size = 0)]
    struct Zst;

    #[derive(Clone, Debug)]
    #[ribbit::pack(size = 32)]
    struct Hole(Zst);

    let zst = Zst;
    let hole = Hole(zst.clone()).pack();
    assert_eq!(zst, hole._0().unpack());
}
