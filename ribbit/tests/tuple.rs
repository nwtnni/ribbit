use core::num::NonZeroU64;

#[test]
fn one_native() {
    #[ribbit::pack(size = 32)]
    #[derive(Copy, Clone)]
    pub struct OneNative(u32);

    let native = OneNative::new(5);
    assert_eq!(native._0(), 5);
    let native = native.with_0(12);
    assert_eq!(native._0(), 12);
}

#[test]
fn one_non_zero() {
    #[ribbit::pack(size = 64, nonzero)]
    #[derive(Copy, Clone)]
    pub struct OneNonZero(NonZeroU64);

    let non_zero = OneNonZero::new(NonZeroU64::MIN);
    assert_eq!(non_zero._0().get(), 1);
    let non_zero = non_zero.with_0(NonZeroU64::new(135).unwrap());
    assert_eq!(non_zero._0().get(), 135);
}
