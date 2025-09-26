#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64)]
struct Foo {
    a: u32,
    b: u64,
}

fn main() {}
