use core::num::NonZeroU8;

use arbitrary_int::u2;

#[ribbit::pack(size = 26)]
#[derive(Copy, Clone)]
pub struct A {
    l: u16,
    m: NonZeroU8,
    c: u2,
}

#[test]
fn check() {
    let a = A::new(15, NonZeroU8::new(10).unwrap(), u2::new(3));
    assert_eq!(format!("{a:?}"), "A { l: 15, m: 10, c: 3 }");
}
