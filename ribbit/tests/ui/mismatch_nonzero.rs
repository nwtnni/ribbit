#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16)]
struct A(u16);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
struct B {
    #[ribbit(size = 16, nonzero)]
    a: A,
    b: u16,
}

fn main() {}
