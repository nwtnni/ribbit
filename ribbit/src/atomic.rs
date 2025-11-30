use core::fmt::Debug;
use core::fmt::Display;
use core::mem::ManuallyDrop;
use core::sync::atomic::Ordering;

use crate::Pack;
use crate::Unpack;

#[repr(transparent)]
pub struct Atomic<T: Pack>(<<T::Packed as Unpack>::Loose as seal::Atomic>::Atomic);

impl<T: Pack> Atomic<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(<<T::Packed as Unpack>::Loose as seal::Atomic>::new(
            Self::packed_to_loose(value.pack()),
        ))
    }

    #[inline]
    pub const fn new_packed(value: T::Packed) -> Self {
        union Transmute<T: Unpack> {
            loose: T::Loose,
            atomic: ManuallyDrop<<T::Loose as seal::Atomic>::Atomic>,
        }

        let loose = Transmute::<T::Packed> {
            loose: Self::packed_to_loose(value),
        };

        unsafe { Self(ManuallyDrop::into_inner(loose.atomic)) }
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> T {
        self.load_packed(ordering).unpack()
    }

    #[inline]
    pub fn load_packed(&self, ordering: Ordering) -> T::Packed {
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as seal::Atomic>::load(
            &self.0, ordering,
        ))
    }

    #[inline]
    pub fn store(&self, value: T, ordering: Ordering) {
        self.store_packed(value.pack(), ordering)
    }

    #[inline]
    pub fn store_packed(&self, value: T::Packed, ordering: Ordering) {
        <<T::Packed as Unpack>::Loose as seal::Atomic>::store(
            &self.0,
            Self::packed_to_loose(value),
            ordering,
        )
    }

    #[inline]
    pub fn get(&mut self) -> T {
        self.get_packed().unpack()
    }

    #[inline]
    pub fn get_packed(&mut self) -> T::Packed {
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as seal::Atomic>::get(
            &mut self.0,
        ))
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.set_packed(value.pack())
    }

    #[inline]
    pub fn set_packed(&mut self, value: T::Packed) {
        <<T::Packed as Unpack>::Loose as seal::Atomic>::set(
            &mut self.0,
            Self::packed_to_loose(value),
        )
    }

    #[inline]
    pub fn compare_exchange(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.compare_exchange_packed(old.pack(), new.pack(), success, failure)
            .map(Unpack::unpack)
            .map_err(Unpack::unpack)
    }

    #[inline]
    pub fn compare_exchange_packed(
        &self,
        old: T::Packed,
        new: T::Packed,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T::Packed, T::Packed> {
        <<T::Packed as Unpack>::Loose as seal::Atomic>::compare_exchange(
            &self.0,
            Self::packed_to_loose(old),
            Self::packed_to_loose(new),
            success,
            failure,
        )
        .map(Self::loose_to_packed)
        .map_err(Self::loose_to_packed)
    }

    #[inline]
    pub fn compare_exchange_weak(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.compare_exchange_weak_packed(old.pack(), new.pack(), success, failure)
            .map(Unpack::unpack)
            .map_err(Unpack::unpack)
    }

    #[inline]
    pub fn compare_exchange_weak_packed(
        &self,
        old: T::Packed,
        new: T::Packed,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T::Packed, T::Packed> {
        <<T::Packed as Unpack>::Loose as seal::Atomic>::compare_exchange_weak(
            &self.0,
            Self::packed_to_loose(old),
            Self::packed_to_loose(new),
            success,
            failure,
        )
        .map(Self::loose_to_packed)
        .map_err(Self::loose_to_packed)
    }

    #[inline]
    pub fn swap(&self, value: T, ordering: Ordering) -> T {
        self.swap_packed(value.pack(), ordering).unpack()
    }

    #[inline]
    pub fn swap_packed(&self, value: T::Packed, ordering: Ordering) -> T::Packed {
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as seal::Atomic>::swap(
            &self.0,
            Self::packed_to_loose(value),
            ordering,
        ))
    }

    #[inline]
    const fn packed_to_loose(value: T::Packed) -> <T::Packed as Unpack>::Loose {
        crate::convert::packed_to_loose(value)
    }

    #[inline]
    const fn loose_to_packed(loose: <T::Packed as Unpack>::Loose) -> T::Packed {
        unsafe { crate::convert::loose_to_packed(loose) }
    }
}

impl<T> Debug for Atomic<T>
where
    T: Pack,
    T::Packed: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<T> Display for Atomic<T>
where
    T: Pack,
    T::Packed: Display,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<T> Default for Atomic<T>
where
    T: Pack,
    T::Packed: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new_packed(T::Packed::default())
    }
}

macro_rules! impl_atomic {
    ($ty:ty, $atomic:ty) => {
        impl super::seal::Atomic for $ty {
            type Atomic = $atomic;

            #[inline]
            fn new(value: Self) -> Self::Atomic {
                <$atomic>::new(value)
            }

            #[inline]
            fn load(atomic: &Self::Atomic, ordering: Ordering) -> Self {
                <$atomic>::load(atomic, ordering)
            }

            #[inline]
            fn store(atomic: &Self::Atomic, value: Self, ordering: Ordering) {
                <$atomic>::store(atomic, value, ordering)
            }

            #[inline]
            fn get(atomic: &mut Self::Atomic) -> Self {
                *<$atomic>::get_mut(atomic)
            }

            #[inline]
            fn set(atomic: &mut Self::Atomic, value: Self) {
                *<$atomic>::get_mut(atomic) = value
            }

            #[inline]
            fn compare_exchange(
                atomic: &Self::Atomic,
                old: Self,
                new: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self, Self> {
                <$atomic>::compare_exchange(atomic, old, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak(
                atomic: &Self::Atomic,
                old: Self,
                new: Self,
                success: Ordering,
                failure: Ordering,
            ) -> Result<Self, Self> {
                <$atomic>::compare_exchange_weak(atomic, old, new, success, failure)
            }

            #[inline]
            fn swap(atomic: &Self::Atomic, value: Self, ordering: Ordering) -> Self {
                <$atomic>::swap(atomic, value, ordering)
            }
        }
    };
}

pub(crate) mod seal {
    use core::sync::atomic::Ordering;

    pub trait Atomic: Sized {
        type Atomic: core::fmt::Debug + Default + Send + Sync;

        fn new(value: Self) -> Self::Atomic;

        fn load(atomic: &Self::Atomic, ordering: Ordering) -> Self;

        fn store(atomic: &Self::Atomic, value: Self, ordering: Ordering);

        fn get(atomic: &mut Self::Atomic) -> Self;

        fn set(atomic: &mut Self::Atomic, value: Self);

        fn compare_exchange(
            atomic: &Self::Atomic,
            old: Self,
            new: Self,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self, Self>;

        fn compare_exchange_weak(
            atomic: &Self::Atomic,
            old: Self,
            new: Self,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self, Self>;

        fn swap(atomic: &Self::Atomic, value: Self, ordering: Ordering) -> Self;
    }

    impl_atomic!(u8, core::sync::atomic::AtomicU8);
    impl_atomic!(u16, core::sync::atomic::AtomicU16);
    impl_atomic!(u32, core::sync::atomic::AtomicU32);
    impl_atomic!(u64, core::sync::atomic::AtomicU64);
}

#[cfg(feature = "portable-u128")]
mod atomic_128 {
    use core::sync::atomic::Ordering;

    impl_atomic!(u128, portable_atomic::AtomicU128);
}

// NOTE: [portable-atomic] currently doesn't support both
// outlining atomics and using vmovdqa for atomic 128 bit loads.
// Work around for now by inlining x86-64 support.
//
// - https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L262-L271
// - https://github.com/taiki-e/portable-atomic/pull/59
#[cfg(not(feature = "portable-u128"))]
mod atomic_128 {
    #[cfg(not(all(
        target_arch = "x86_64",
        target_pointer_width = "64",
        target_feature = "avx",
        target_feature = "cmpxchg16b",
    )))]
    compile_error!(
        "Atomic u128 only implemented for x86-64 + avx + cmpxchg16b; enable portable-u128 feature for other targets"
    );

    use core::arch::asm;
    use core::arch::x86_64::__m128i;
    use core::cell::UnsafeCell;
    use core::sync::atomic::Ordering;

    impl_atomic!(u128, AtomicU128);

    #[repr(transparent)]
    pub struct AtomicU128(UnsafeCell<__m128i>);

    impl Default for AtomicU128 {
        fn default() -> Self {
            Self(UnsafeCell::new(unsafe {
                core::mem::transmute::<u128, __m128i>(0)
            }))
        }
    }

    impl core::fmt::Debug for AtomicU128 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.load(Ordering::Relaxed).fmt(f)
        }
    }

    unsafe impl Send for AtomicU128 {}
    unsafe impl Sync for AtomicU128 {}

    impl AtomicU128 {
        #[inline]
        pub(super) const fn new(value: u128) -> Self {
            Self(UnsafeCell::new(unsafe {
                core::mem::transmute::<u128, __m128i>(value)
            }))
        }

        #[inline]
        pub(super) const fn get_mut(&mut self) -> &mut u128 {
            unsafe { core::mem::transmute(self.0.get_mut()) }
        }

        // https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L158-L175
        #[inline]
        pub(super) fn load(&self, _ordering: Ordering) -> u128 {
            unsafe {
                let output: __m128i;
                asm! {
                    "vmovdqa {output}, xmmword ptr [{address}]",
                    address = in(reg) self.0.get(),
                    output = out(xmm_reg) output,
                    options(nostack, preserves_flags, readonly),
                }
                core::mem::transmute::<__m128i, u128>(output)
            }
        }

        // https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L180-L218
        #[inline]
        pub(super) fn store(&self, value: u128, ordering: Ordering) {
            unsafe {
                let input = core::mem::transmute::<u128, __m128i>(value);
                match ordering {
                    Ordering::Relaxed | Ordering::Release => {
                        asm!(
                            "vmovdqa xmmword ptr [{address}], {input}",
                            address = in(reg) self.0.get(),
                            input = in(xmm_reg) input,
                            options(nostack, preserves_flags),
                        );
                    }
                    Ordering::SeqCst => {
                        let mut uninit = core::mem::MaybeUninit::<u64>::uninit();
                        asm!(
                            concat!("vmovdqa xmmword ptr [{address}], {input}"),
                            concat!("xchg qword ptr [{uninit}], {any}"),
                            address = in(reg) self.0.get(),
                            input = in(xmm_reg) input,
                            uninit = inout(reg) uninit.as_mut_ptr() => _,
                            any = lateout(reg) _,
                            options(nostack, preserves_flags),
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }

        // https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L95-L142
        #[inline]
        pub(super) fn compare_exchange(
            &self,
            old: u128,
            new: u128,
            _success: Ordering,
            _failure: Ordering,
        ) -> Result<u128, u128> {
            unsafe {
                let mut success: u8;
                let old = U128 { whole: old };
                let new = U128 { whole: new };
                let lo;
                let hi;

                asm!(
                    "xchg {save_rbx}, rbx",
                    "lock cmpxchg16b xmmword ptr [rdi]",
                    "sete cl",
                    "mov rbx, {save_rbx}",
                    save_rbx = inout(reg) new.pair.lo => _,
                    in("rcx") new.pair.hi,
                    inout("rax") old.pair.lo => lo,
                    inout("rdx") old.pair.hi => hi,
                    in("rdi") self.0.get(),
                    lateout("cl") success,
                    options(nostack),
                );

                let out = U128 {
                    pair: Pair { lo, hi },
                }
                .whole;

                core::hint::assert_unchecked(success == 0 || success == 1);
                if success == 0 {
                    Err(out)
                } else {
                    Ok(out)
                }
            }
        }

        #[inline]
        pub(super) fn compare_exchange_weak(
            &self,
            old: u128,
            new: u128,
            success: Ordering,
            failure: Ordering,
        ) -> Result<u128, u128> {
            self.compare_exchange(old, new, success, failure)
        }

        // https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L442-L495
        #[inline]
        pub(super) fn swap(&self, value: u128, _order: Ordering) -> u128 {
            unsafe {
                let value = U128 { whole: value };
                let lo;
                let hi;

                asm!(
                    "xchg {save_rbx}, rbx",
                    "mov rax, qword ptr [rdi]",
                    "mov rdx, qword ptr [rdi + 8]",
                    "2:",
                        "lock cmpxchg16b xmmword ptr [rdi]",
                        "jne 2b",
                    "mov rbx, {save_rbx}",
                    save_rbx = inout(reg) value.pair.lo => _,
                    in("rcx") value.pair.hi,
                    out("rax") lo,
                    out("rdx") hi,
                    in("rdi") self.0.get(),
                    options(nostack),
                );

                U128 {
                    pair: Pair { lo, hi },
                }
                .whole
            }
        }
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    union U128 {
        whole: u128,
        pair: Pair,
    }

    // https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/utils.rs#L393-L414
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Pair {
        #[cfg(target_endian = "little")]
        lo: u64,
        hi: u64,
        #[cfg(target_endian = "big")]
        lo: u64,
    }
}
