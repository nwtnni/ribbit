#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 16)]
struct A {
    #[ribbit(size = 3)]
    lo: u8,

    #[ribbit(size = 13)]
    hi: u8,
}

fn main() {}
