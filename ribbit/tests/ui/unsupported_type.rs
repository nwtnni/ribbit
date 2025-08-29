#[ribbit::pack(size = 24)]
#[derive(Clone)]
struct Bad {
    a: (u8, u16),
}

fn main() {}
