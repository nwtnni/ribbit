use core::num::NonZeroU16;

#[derive(Copy, Clone)]
#[ribbit::pack(size = 16, nonzero)]
struct Inner(NonZeroU16);

#[ribbit::pack(size = 21)]
struct Outer {
    pad: u5,
    inner: Inner,
}

fn main() {}
