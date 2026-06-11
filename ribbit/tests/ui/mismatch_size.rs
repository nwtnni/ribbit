#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16)]
struct A(u16);

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16)]
struct B {
    #[ribbit(size = 1)]
    a: crate::A,
    b: ribbit::u15,
}

fn main() {}
