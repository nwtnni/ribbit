#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
struct A(u16);

#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
struct B {
    #[ribbit(size = 1)]
    a: A,
    b: u15,
}

fn main() {}
