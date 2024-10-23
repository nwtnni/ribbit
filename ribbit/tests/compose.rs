#[test]
fn basic() {
    #[ribbit::pack(size = 32)]
    #[derive(Copy, Clone)]
    struct Low {
        a: u32,
    }

    #[derive(Copy, Clone)]
    #[ribbit::pack(size = 64)]
    struct Whole {
        #[ribbit(size = 32)]
        low: Low,
        b: u32,
    }
}
