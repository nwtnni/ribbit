use core::marker::PhantomData;
use core::num::NonZeroU64;

#[test]
fn custom_zst() {
    struct Foo;

    #[ribbit::pack(size = 64)]
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

    let h = Phantom::<usize>::new(NonZeroU64::new(0xdead_beef).unwrap());
    assert_eq!(h.value.get(), 0xdead_beef);
    assert_eq!(h.a().get(), 0xdead_beef);
}

#[test]
fn pack_zst() {
    #[ribbit::pack(size = 0, debug, eq)]
    struct Foo;

    let zst = Foo::new();

    #[allow(clippy::let_unit_value)]
    let packed = ribbit::private::pack(zst);
    let unpacked = unsafe { ribbit::private::unpack::<Foo>(packed) };

    assert_eq!(zst, unpacked);
}

#[test]
fn pack_zst_large() {
    #[ribbit::pack(size = 0, debug, eq)]
    struct Zst;

    #[ribbit::pack(size = 32, debug, eq)]
    struct Hole(Zst);

    let zst = Zst::new();
    let hole = Hole::new(zst);
    assert_eq!(zst, hole._0());
}
