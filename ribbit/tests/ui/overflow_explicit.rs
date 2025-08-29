#[ribbit::pack(size = 32)]
#[derive(Clone)]
struct Foo {
    #[ribbit(offset = 1)]
    a: u32,
}

fn main() {}
