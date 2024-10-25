#[ribbit::pack(size = 32)]
#[derive(Copy, Clone)]
pub struct OneNative(u32);

#[ribbit::pack(size = 64, nonzero)]
#[derive(Copy, Clone)]
pub struct OneNonZero(NonZeroU64);
