use core::marker::PhantomData;

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
