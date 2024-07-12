#[test]
fn basic() {
    #[ribbit::pack(size = 64)]
    #[derive(Debug)]
    struct Half {
        a: u32,
        b: u32,
    }

    let h = Half { value: 0xdeadbeef };
    println!("{:?}", h);
}
