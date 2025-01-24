#[ribbit::pack(size = 16)]
struct A(u16);

#[ribbit::pack(size = 32)]
struct B {
    #[ribbit(size = 16, nonzero)]
    a: A,
    b: u16,
}

fn main() {}
