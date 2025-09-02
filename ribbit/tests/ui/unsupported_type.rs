#[ribbit::pack(size = 24)]
#[derive(Copy, Clone)]
struct Bad {
    a: (u8, u16),
}

fn main() {}
