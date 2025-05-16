use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::sync::atomic::AtomicU16;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;

use crate::Pack;

macro_rules! atomic {
    ($name:ident, $atomic:ty, $loose:ty, $size:expr) => {
        #[repr(transparent)]
        pub struct $name<T> {
            value: $atomic,
            _type: PhantomData<T>,
        }

        impl<T> Debug for $name<T>
        where
            T: Debug + Pack,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.load(Ordering::Relaxed).fmt(f)
            }
        }

        impl<T> Display for $name<T>
        where
            T: Display + Pack,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.load(Ordering::Relaxed).fmt(f)
            }
        }

        impl<T> $name<T>
        where
            T: Pack,
        {
            const INVARIANT: () = assert!(T::BITS <= $size);

            pub const fn new(value: T) -> Self {
                const { Self::INVARIANT }
                Self {
                    value: <$atomic>::new(Self::loosen(value)),
                    _type: PhantomData,
                }
            }

            pub fn load(&self, ordering: Ordering) -> T {
                Self::pack(self.value.load(ordering))
            }

            pub fn store(&self, value: T, ordering: Ordering) {
                self.value.store(Self::loosen(value), ordering)
            }

            pub fn compare_exchange(
                &self,
                old: T,
                new: T,
                success: Ordering,
                failure: Ordering,
            ) -> Result<T, T> {
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
                self.value
                    .compare_exchange_weak(Self::loosen(old), Self::loosen(new), success, failure)
                    .map(Self::pack)
                    .map_err(Self::pack)
            }

            pub fn swap(&self, value: T, ordering: Ordering) -> T {
                Self::pack(self.value.swap(Self::loosen(value), ordering))
            }

            const fn loosen(value: T) -> $loose {
                const { Self::INVARIANT }
                let loose = crate::convert::packed_to_loose(value);
                crate::convert::loose_to_loose(loose)
            }

            const fn pack(loose: $loose) -> T {
                const { Self::INVARIANT }
                let loose = crate::convert::loose_to_loose(loose);
                unsafe { crate::convert::loose_to_packed(loose) }
            }
        }
    };
}

atomic!(A8, AtomicU8, u8, 8);
atomic!(A16, AtomicU16, u16, 16);
atomic!(A32, AtomicU32, u32, 32);
atomic!(A64, AtomicU64, u64, 64);

#[cfg(feature = "atomic-u128")]
portable_atomic::cfg_has_atomic_128! {
    atomic!(A128, ::portable_atomic::AtomicU128, u128, 128);
}

#[cfg(feature = "atomic-u128")]
portable_atomic::cfg_no_atomic_128! {
    compile_error!(
        "atomic-u128 feature enabled, \
        but not supported by target: \
        maybe missing target feature? \
        (https://github.com/taiki-e/portable-atomic/blob/8bd3b5d267a69d37ab31b74ceb845a8345c1eaf5/src/imp/atomic128/README.md)");
}
