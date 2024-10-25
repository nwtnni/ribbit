#[ribbit::pack(size = 1)]
#[derive(Copy, Clone)]
struct A(bool);

#[test]
fn single() {
    let a = A::new(true);
    assert!(a._0());
}

#[ribbit::pack(size = 2)]
#[derive(Copy, Clone)]
struct B(bool, bool);

#[test]
fn multiple() {
    let b = B::new(true, false);
    assert!(b._0());
    assert!(!b._1());
}
