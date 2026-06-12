use core::num::NonZeroU64;

use ribbit::Pack as _;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32)]
pub struct OneNative(u32);

#[test]
fn one_native() {
    let native = OneNative(5).pack();
    assert_eq!(native._0(), 5);
    let native = native.with_0(12);
    assert_eq!(native._0(), 12);
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 64, nonzero)]
pub struct OneNonZero(NonZeroU64);

#[test]
fn one_non_zero() {
    let non_zero = OneNonZero(NonZeroU64::MIN).pack();
    assert_eq!(non_zero._0().get(), 1);
    let non_zero = non_zero.with_0(NonZeroU64::new(135).unwrap());
    assert_eq!(non_zero._0().get(), 135);
}
