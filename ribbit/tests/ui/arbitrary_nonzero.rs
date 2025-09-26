use std::num::NonZeroU8;

use ribbit::u7;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 15, nonzero)]
struct Bad {
    a: u7,
    b: NonZeroU8,
}

fn main() {}
