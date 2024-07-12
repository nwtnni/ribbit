#[test]
fn basic() {
    #[ribbit::pack(size = 64)]
    struct Half {
        a: u32,
        b: u32,
    }
}
