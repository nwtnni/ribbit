use std::num::NonZeroU16;

use ribbit::u5;

#[ribbit::pack(size = 16, nonzero)]
struct Inner(NonZeroU16);

#[ribbit::pack(size = 21)]
struct Outer {
    pad: u5,
    inner: Inner,
}

fn main() {}
