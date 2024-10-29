#[ribbit::pack(size = 48)]
#[derive(Copy, Clone)]
struct Versioned<T> {
    version: u16,
    #[ribbit(size = 32)]
    inner: T,
}

#[ribbit::pack(size = 32)]
#[derive(Copy, Clone)]
struct A(u32);

#[ribbit::pack(size = 32)]
#[derive(Copy, Clone)]
struct B {
    hi: u16,
    lo: u16,
}

#[test]
fn basic() {
    let a = Versioned::new(15, A::new(36));

    assert_eq!(a.version(), 15);
    assert_eq!(a.inner()._0(), 36);

    let a = a.with_version(99);

    assert_eq!(a.version(), 99);
    assert_eq!(a.inner()._0(), 36);

    let a = a.with_inner(A::new(52));

    assert_eq!(a.version(), 99);
    assert_eq!(a.inner()._0(), 52);
}

#[test]
fn compose() {
    let b = Versioned::new(15, B::new(1, 2));

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
