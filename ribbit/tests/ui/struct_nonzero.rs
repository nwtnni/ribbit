#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32, nonzero)]
struct Bad(u32);

fn main() {}
