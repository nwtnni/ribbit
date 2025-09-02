use ribbit::Pack as _;

#[derive(Copy, Clone)]
#[ribbit::pack(size = 1)]
struct A(bool);

#[test]
fn single() {
    let a = A(true).pack();
    assert!(a._0());
}

#[derive(Copy, Clone)]
#[ribbit::pack(size = 2)]
struct B(bool, bool);

#[test]
fn multiple() {
    let b = B(true, false).pack();
    assert!(b._0());
    assert!(!b._1());
}
