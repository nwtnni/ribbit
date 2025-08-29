use ribbit::u15;

#[ribbit::pack(size = 16)]
#[derive(Clone)]
struct A(u16);

#[ribbit::pack(size = 16)]
#[derive(Clone)]
struct B {
    #[ribbit(size = 1)]
    a: A,
    b: u15,
}

fn main() {}
