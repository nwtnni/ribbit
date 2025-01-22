#[ribbit::pack(size = 16)]
struct A(u16);

#[ribbit::pack(size = 16)]
struct B {
    #[ribbit(size = 1)]
    a: A,
    b: u15,
}

fn main() {}
