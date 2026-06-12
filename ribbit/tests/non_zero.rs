use core::num::NonZeroU16;
use core::num::NonZeroU32;

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 32, non_zero)]
struct Aligned(NonZeroU32);

#[test]
fn niche_aligned() {
    assert_eq!(
        core::mem::size_of::<ribbit::Packed<Aligned>>(),
        core::mem::size_of::<Option<ribbit::Packed<Aligned>>>(),
    );
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 24, non_zero)]
struct Unaligned(NonZeroU16, u8);

#[test]
fn niche_unaligned() {
    assert_eq!(
        core::mem::size_of::<ribbit::Packed<Unaligned>>(),
        core::mem::size_of::<Option<ribbit::Packed<Unaligned>>>(),
    );
}

#[derive(ribbit::Pack, Copy, Clone)]
#[ribbit(size = 56, non_zero)]
struct ComposeUnaligned {
    #[ribbit(size = 24, non_zero)]
    lo: crate::Unaligned,
    #[ribbit(size = 32, non_zero)]
    hi: crate::Aligned,
}

#[test]
fn niche_compose_unaligned() {
    assert_eq!(
        core::mem::size_of::<ribbit::Packed<ComposeUnaligned>>(),
        core::mem::size_of::<Option<ribbit::Packed<ComposeUnaligned>>>(),
    );
}
