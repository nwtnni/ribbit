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
            T: Pack,
            T::Packed: Debug,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                self.load(Ordering::Relaxed).fmt(f)
            }
        }

        impl<T> Display for $name<T>
        where
            T: Pack,
            T::Packed: Display,
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

            pub const fn new(value: T::Packed) -> Self {
                const { Self::INVARIANT }
                Self {
                    value: <$atomic>::new(Self::loosen(value)),
                    _type: PhantomData,
                }
            }

            pub fn load(&self, ordering: Ordering) -> T::Packed {
                Self::pack(self.value.load(ordering))
            }

            pub fn store(&self, value: T::Packed, ordering: Ordering) {
                self.value.store(Self::loosen(value), ordering)
            }

            pub fn get(&mut self) -> T::Packed {
                Self::pack(*self.value.get_mut())
            }

            pub fn set(&mut self, value: T::Packed) {
                *self.value.get_mut() = Self::loosen(value);
            }

            pub fn compare_exchange(
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

            pub fn swap(&self, value: T::Packed, ordering: Ordering) -> T::Packed {
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
portable_atomic::cfg_has_atomic_128! {
    atomic!(Atomic128, ::portable_atomic::AtomicU128, u128, 128);
}

#[cfg(feature = "atomic-u128")]
portable_atomic::cfg_no_atomic_128! {
    compile_error!(
        "atomic-u128 feature enabled, \
        but not supported by target: \
        maybe missing target feature? \
        (https://github.com/taiki-e/portable-atomic/blob/8bd3b5d267a69d37ab31b74ceb845a8345c1eaf5/src/imp/atomic128/README.md)");
}
