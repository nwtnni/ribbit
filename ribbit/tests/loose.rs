use std::collections::BTreeSet;
use std::collections::HashSet;

#[ribbit::pack(size = 32, hash, debug, eq, ord)]
struct Wrap<T> {
    #[ribbit(size = 32)]
    inner: T,
}

#[test]
fn hash() {
    let mut set = HashSet::new();
    let a = Wrap::new(5u32);
    set.insert(a);
    assert!(set.contains(&a));
}

#[test]
fn ord() {
    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = BTreeSet::new();
    let a = Wrap::new(1u32);
    let b = Wrap::new(2u32);
    set.insert(a);
    set.insert(b);
    assert!(set.contains(&a));
    assert!(set.contains(&b));
    assert_eq!(set.pop_last(), Some(b));
    assert_eq!(set.pop_first(), Some(a));
}

#[test]
fn loose_derive() {
    #[ribbit::pack(size = 32)]
    struct Inner {
        lo: u16,
        hi: u16,
    }

    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = HashSet::new();
    let a = Wrap::new(Inner::new(5, 7));
    set.insert(a);
    assert!(set.contains(&a));
}
