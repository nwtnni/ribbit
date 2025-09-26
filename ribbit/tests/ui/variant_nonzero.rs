#[repr(u8)]
#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16, nonzero)]
enum Foo {
    #[ribbit(size = 8)]
    Bar(u8) = 1,
    #[ribbit(size = 8)]
    Baz(u8) = 0,
}

fn main() {}
