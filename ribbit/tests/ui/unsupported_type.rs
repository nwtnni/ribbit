#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 24)]
struct Bad {
    a: (u8, u16),
}

fn main() {}
