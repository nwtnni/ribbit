#[ribbit::pack(size = 1)]
struct A(bool);

#[test]
fn single() {
    let a = A::new(true);
    assert!(a._0());
}

#[ribbit::pack(size = 2)]
struct B(bool, bool);

#[test]
fn multiple() {
    let b = B::new(true, false);
    assert!(b._0());
    assert!(!b._1());
}
