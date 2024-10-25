#[ribbit::pack(size = 32)]
struct Foo {
    #[ribbit(offset = 1)]
    a: u32,
}

fn main() {}
