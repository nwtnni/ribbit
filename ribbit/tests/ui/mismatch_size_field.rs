#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
struct A {
    #[ribbit(size = 3)]
    lo: u8,

    #[ribbit(size = 13)]
    hi: u8,
}

fn main() {}
