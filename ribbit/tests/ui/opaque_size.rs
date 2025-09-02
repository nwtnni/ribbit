use std::num::NonZeroU16;

#[ribbit::pack(size = 16, nonzero)]
#[derive(Copy, Clone)]
struct Inner(NonZeroU16);

#[ribbit::pack(size = 21)]
#[derive(Copy, Clone)]
struct Outer {
    pad: ribbit::u5,
    inner: Inner,
}

fn main() {}
