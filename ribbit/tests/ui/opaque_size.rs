use std::num::NonZeroU16;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16, nonzero)]
struct Inner(NonZeroU16);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 21)]
struct Outer {
    pad: ribbit::u5,
    inner: Inner,
}

fn main() {}
