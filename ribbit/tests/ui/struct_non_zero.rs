#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32, non_zero)]
struct Bad(u32);

fn main() {}
