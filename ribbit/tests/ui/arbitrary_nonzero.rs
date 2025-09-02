use std::num::NonZeroU8;

use ribbit::u7;

#[ribbit::pack(size = 15, nonzero)]
#[derive(Copy, Clone)]
struct Bad {
    a: u7,
    b: NonZeroU8,
}

fn main() {}
