#[ribbit::pack(size = 16, nonzero)]
#[derive(Copy, Clone)]
enum Foo {
    #[ribbit(size = 8)]
    Bar(u8) = 1,
    #[ribbit(size = 8)]
    Baz(u8) = 0,
}

fn main() {}
