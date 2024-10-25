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

#[test]
fn nonzero() {
    #[ribbit::pack(size = 16, nonzero)]
    #[derive(Copy, Clone)]
    struct Low {
        a: NonZeroU16,
    }

    #[ribbit::pack(size = 48)]
    #[derive(Copy, Clone)]
    struct Whole {
        #[ribbit(size = 16, nonzero)]
        low: Low,
        b: u32,
    }
}
