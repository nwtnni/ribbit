use ribbit::u2;
use ribbit::Pack as _;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 1)]
struct A(bool);

#[test]
fn single() {
    let a = A(true).pack();
    assert!(a._0());
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 2)]
struct B(bool, bool);

#[test]
fn multiple() {
    let b = B(true, false).pack();
    assert!(b._0());
    assert!(!b._1());
}

#[test]
fn unchecked() {
    let b = unsafe { ribbit::Packed::<B>::new_unchecked(u2::new(0b10)) };
    assert!(!b._0());
    assert!(b._1());
}
