#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
struct A(u16);

#[ribbit::pack(size = 32)]
#[derive(Copy, Clone)]
struct B {
    #[ribbit(size = 16, nonzero)]
    a: A,
    b: u16,
}

fn main() {}
