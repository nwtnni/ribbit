#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16, nonzero)]
struct Inner(ribbit::NonZeroU16);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 21)]
struct Outer {
    pad: ribbit::u5,
    inner: Inner,
}

fn main() {}
