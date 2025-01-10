use core::marker::PhantomData;
use core::num::NonZeroU64;

#[test]
fn custom_zst() {
    struct Foo;

    #[ribbit::pack(size = 64)]
    #[derive(Copy, Clone)]
    struct S {
        a: u64,
        #[ribbit(size = 0)]
        foo: Foo,
    }

    let h = S::new(0xdead_beef);
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

    let h = Phantom::<usize>::new(0xdead_beef);
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

    let h = Phantom::<usize>::new(NonZeroU64::new(0xdead_beef).unwrap());
    assert_eq!(h.value.get(), 0xdead_beef);
    assert_eq!(h.a().get(), 0xdead_beef);
}

#[test]
fn pack_zst() {
    #[ribbit::pack(size = 0, debug)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    struct Foo;

    let zst = Foo::new();

    #[allow(clippy::let_unit_value)]
    let packed = ribbit::private::pack(zst);
    let unpacked = unsafe { ribbit::private::unpack::<Foo>(packed) };

    assert_eq!(zst, unpacked);
}
