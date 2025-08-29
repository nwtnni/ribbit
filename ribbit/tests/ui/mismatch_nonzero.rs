#[ribbit::pack(size = 16)]
#[derive(Clone)]
struct A(u16);

#[ribbit::pack(size = 32)]
#[derive(Clone)]
struct B {
    #[ribbit(size = 16, nonzero)]
    a: A,
    b: u16,
}

fn main() {}
