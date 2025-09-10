use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::sync::atomic::AtomicU16;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;

use crate::Pack;
use crate::Unpack;

macro_rules! atomic {
    ($name:ident, $atomic:ty, $loose:ty, $size:expr) => {
        #[repr(transparent)]
        pub struct $name<T> {
            value: $atomic,
            _type: PhantomData<T>,
        }

        impl<T> Debug for $name<T>
        where
            T: Pack + Debug,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.load(Ordering::Relaxed).fmt(f)
            }
        }

        impl<T> Display for $name<T>
        where
            T: Pack + Display,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.load(Ordering::Relaxed).fmt(f)
            }
        }

        impl<T> $name<T>
        where
            T: Pack,
        {
            const INVARIANT: () = assert!(<<T as Pack>::Packed as Unpack>::BITS <= $size);

            pub fn new(value: T) -> Self {
                const { Self::INVARIANT }
                Self::from_packed(value.pack())
            }

            pub const fn from_packed(value: T::Packed) -> Self {
                const { Self::INVARIANT }
                Self {
                    value: <$atomic>::new(Self::loosen(value)),
                    _type: PhantomData,
                }
            }

            pub fn load(&self, ordering: Ordering) -> T {
                self.load_packed(ordering).unpack()
            }

            pub fn load_packed(&self, ordering: Ordering) -> T::Packed {
                Self::pack(self.value.load(ordering))
            }

            pub fn store(&self, value: T, ordering: Ordering) {
                self.store_packed(value.pack(), ordering)
            }

            pub fn store_packed(&self, value: T::Packed, ordering: Ordering) {
                self.value.store(Self::loosen(value), ordering)
            }

            pub fn get(&mut self) -> T {
                self.get_packed().unpack()
            }

            pub fn get_packed(&mut self) -> T::Packed {
                Self::pack(*self.value.get_mut())
            }

            pub fn set(&mut self, value: T) {
                self.set_packed(value.pack())
            }

            pub fn set_packed(&mut self, value: T::Packed) {
                *self.value.get_mut() = Self::loosen(value);
            }

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

            pub fn compare_exchange_packed(
                &self,
                old: T::Packed,
                new: T::Packed,
                success: Ordering,
                failure: Ordering,
            ) -> Result<T::Packed, T::Packed> {
                self.value
                    .compare_exchange(Self::loosen(old), Self::loosen(new), success, failure)
                    .map(Self::pack)
                    .map_err(Self::pack)
            }

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

            pub fn compare_exchange_weak_packed(
                &self,
                old: T::Packed,
                new: T::Packed,
                success: Ordering,
                failure: Ordering,
            ) -> Result<T::Packed, T::Packed> {
                self.value
                    .compare_exchange_weak(Self::loosen(old), Self::loosen(new), success, failure)
                    .map(Self::pack)
                    .map_err(Self::pack)
            }

            pub fn swap(&self, value: T, ordering: Ordering) -> T {
                self.swap_packed(value.pack(), ordering).unpack()
            }

            pub fn swap_packed(&self, value: T::Packed, ordering: Ordering) -> T::Packed {
                Self::pack(self.value.swap(Self::loosen(value), ordering))
            }

            const fn loosen(value: T::Packed) -> $loose {
                const { Self::INVARIANT }
                let loose = crate::convert::packed_to_loose(value);
                crate::convert::loose_to_loose(loose)
            }

            const fn pack(loose: $loose) -> T::Packed {
                const { Self::INVARIANT }
                let loose = crate::convert::loose_to_loose(loose);
                unsafe { crate::convert::loose_to_packed(loose) }
            }
        }
    };
}

atomic!(Atomic8, AtomicU8, u8, 8);
atomic!(Atomic16, AtomicU16, u16, 16);
atomic!(Atomic32, AtomicU32, u32, 32);
atomic!(Atomic64, AtomicU64, u64, 64);

#[cfg(feature = "atomic-u128")]
atomic!(Atomic128, atomic_128::AtomicU128, u128, 128);

#[cfg(feature = "atomic-u128")]
// NOTE: [portable-atomic] currently doesn't support both
// outlining atomics and using vmovdqa for atomic 128 bit loads.
// Work around for now by inlining x86-64 support.
//
// - https://github.com/taiki-e/portable-atomic/blob/d118cf01f852ef4cc1fa4e0a08f80f5e2a5c8f41/src/imp/atomic128/x86_64.rs#L262-L271
// - https://github.com/taiki-e/portable-atomic/pull/59
mod atomic_128 {
    #[cfg(not(all(
        target_arch = "x86_64",
        target_pointer_width = "64",
        target_feature = "avx",
        target_feature = "cmpxchg16b",
    )))]
    compile_error!(
        "Unsupported target feature set for atomic-u128 feature: avx and cmpxchg16b required"
    );

    use core::arch::asm;
    use core::arch::x86_64::__m128i;
    use core::cell::UnsafeCell;
    use core::sync::atomic::Ordering;

    #[repr(transparent)]
    pub(super) struct AtomicU128(UnsafeCell<__m128i>);

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
                    Ok(out)
                } else {
                    Err(out)
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
