#[ribbit::pack(size = 16)]
#[derive(Copy, Clone)]
enum Named {
    #[ribbit(size = 16)]
    #[derive(Copy, Clone)]
    A { a: u16 },
}
