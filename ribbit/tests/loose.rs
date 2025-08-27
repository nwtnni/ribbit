use std::collections::BTreeSet;
use std::collections::HashSet;

use ribbit::Pack as _;

#[derive(Clone, Debug)]
#[ribbit::pack(size = 32, hash, debug, eq, ord)]
struct Wrap<T> {
    #[ribbit(size = 32)]
    inner: T,
}

#[test]
fn hash() {
    let mut set = HashSet::new();
    let a = Wrap { inner: 5u32 }.pack();
    set.insert(a);
    assert!(set.contains(&a));
}

#[test]
fn ord() {
    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = BTreeSet::new();
    let a = Wrap { inner: 1u32 }.pack();
    let b = Wrap { inner: 2u32 }.pack();
    set.insert(a);
    set.insert(b);
    assert!(set.contains(&a));
    assert!(set.contains(&b));
    assert_eq!(set.pop_last(), Some(b));
    assert_eq!(set.pop_first(), Some(a));
}

#[test]
fn loose_derive() {
    #[derive(Clone)]
    #[ribbit::pack(size = 32)]
    struct Inner {
        lo: u16,
        hi: u16,
    }

    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = HashSet::new();
    let a = Wrap {
        inner: Inner { lo: 5, hi: 7 },
    }
    .pack();
    set.insert(a);
    assert!(set.contains(&a));
}
