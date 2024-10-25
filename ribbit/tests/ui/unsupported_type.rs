#[ribbit::pack(size = 24)]
struct Bad {
    a: (u8, u16),
}

fn main() {}
