use std::collections::BTreeSet;
use std::collections::HashSet;

use ribbit::Pack as _;

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 32, derive(Hash, Debug, Eq, Ord))]
struct Outer<T> {
    #[ribbit(size = 32)]
    inner: T,
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
struct Inner {
    lo: u16,
    hi: u16,
}

#[test]
fn hash() {
    let mut set = HashSet::new();
    let a = Outer { inner: 5u32 }.pack();
    set.insert(a);
    assert!(set.contains(&a));
}

#[test]
fn ord() {
    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = BTreeSet::new();
    let a = Outer { inner: 1u32 }.pack();
    let b = Outer { inner: 2u32 }.pack();
    set.insert(a);
    set.insert(b);
    assert!(set.contains(&a));
    assert!(set.contains(&b));
    assert_eq!(set.pop_last(), Some(b));
    assert_eq!(set.pop_first(), Some(a));
}

#[test]
fn loose_derive() {
    // `Wrap<Inner>: Hash + Eq even though Inner: !Hash + !Eq
    let mut set = HashSet::new();
    let a = Outer {
        inner: Inner { lo: 5, hi: 7 },
    }
    .pack();
    set.insert(a);
    assert!(set.contains(&a));
}
