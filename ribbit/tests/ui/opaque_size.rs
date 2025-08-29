use std::num::NonZeroU16;

#[ribbit::pack(size = 16, nonzero)]
#[derive(Clone)]
struct Inner(NonZeroU16);

#[ribbit::pack(size = 21)]
#[derive(Clone)]
struct Outer {
    pad: ribbit::u5,
    inner: Inner,
}

fn main() {}
