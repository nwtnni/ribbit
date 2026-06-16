use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use crate::Loose;
use crate::Pack;
use crate::Unpack;

#[doc(no_inline)]
pub use core::sync::atomic::AtomicU16;
#[doc(no_inline)]
pub use core::sync::atomic::AtomicU32;
#[doc(no_inline)]
pub use core::sync::atomic::AtomicU64;
#[doc(no_inline)]
pub use core::sync::atomic::AtomicU8;
#[cfg(feature = "u128")]
#[doc(no_inline)]
pub use portable_atomic::AtomicU128;

/// Type-safe atomic wrapper for types implementing [`Pack`] and [`Unpack`].
///
/// Generic type parameter `R` defaults to standard library and `portable_atomic`
/// atomic integer types, but can be overridden.
#[repr(transparent)]
pub struct Atomic<T, R = <<<T as Pack>::Packed as Unpack>::Loose as Loose>::Atomic> {
    raw: R,
    unpacked: PhantomData<T>,
}

impl<T, R> Atomic<T, R> {
    #[inline]
    pub const fn from_raw(raw: R) -> Self {
        Self {
            raw,
            unpacked: PhantomData,
        }
    }
}

impl<T, R> Atomic<T, R>
where
    T: Pack,
    R: Raw<<<T as Pack>::Packed as Unpack>::Loose>,
{
    #[inline]
    pub fn new(unpacked: T) -> Self {
        Self::new_packed(unpacked.pack())
    }

    #[inline]
    pub fn new_packed(packed: T::Packed) -> Self {
        Self::from_raw(R::new_(Self::packed_to_loose(packed)))
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> T {
        self.load_packed(ordering).unpack()
    }

    #[inline]
    pub fn load_packed(&self, ordering: Ordering) -> T::Packed {
        Self::loose_to_packed(R::load_(&self.raw, ordering))
    }

    #[inline]
    pub fn store(&self, value: T, ordering: Ordering) {
        self.store_packed(value.pack(), ordering)
    }

    #[inline]
    pub fn store_packed(&self, value: T::Packed, ordering: Ordering) {
        R::store_(&self.raw, Self::packed_to_loose(value), ordering)
    }

    #[inline]
    pub fn get(&mut self) -> T {
        self.get_packed().unpack()
    }

    #[inline]
    pub fn get_packed(&mut self) -> T::Packed {
        Self::loose_to_packed(R::get_(&mut self.raw))
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.set_packed(value.pack())
    }

    #[inline]
    pub fn set_packed(&mut self, value: T::Packed) {
        R::set_(&mut self.raw, Self::packed_to_loose(value))
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
        R::compare_exchange_(
            &self.raw,
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
        R::compare_exchange_weak_(
            &self.raw,
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
        Self::loose_to_packed(R::swap_(&self.raw, Self::packed_to_loose(value), ordering))
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

impl<T, R> Clone for Atomic<T, R>
where
    R: Clone,
{
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            unpacked: PhantomData,
        }
    }
}

impl<T, R> Debug for Atomic<T, R>
where
    T: Pack,
    T::Packed: Debug,
    R: Raw<<<T as Pack>::Packed as Unpack>::Loose>,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<T, R> Display for Atomic<T, R>
where
    T: Pack,
    T::Packed: Display,
    R: Raw<<<T as Pack>::Packed as Unpack>::Loose>,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<T, R> Default for Atomic<T, R>
where
    R: Default,
{
    #[inline]
    fn default() -> Self {
        Self::from_raw(R::default())
    }
}

impl<T, R> From<R> for Atomic<T, R> {
    #[inline]
    fn from(raw: R) -> Self {
        Self::from_raw(raw)
    }
}

/// Interface for underlying atomic integer.
pub trait Raw<T>: core::fmt::Debug + Default + Send + Sync {
    fn new_(loose: T) -> Self;

    fn load_(&self, ordering: Ordering) -> T;

    fn store_(&self, value: T, ordering: Ordering);

    fn get_(&mut self) -> T;

    fn set_(&mut self, value: T);

    fn compare_exchange_(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    fn compare_exchange_weak_(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    fn swap_(&self, value: T, ordering: Ordering) -> T;
}

/// Convenience macro for implementing [`Raw`].
#[macro_export]
macro_rules! impl_raw {
    ($loose:ty, $atomic:ty) => {
        impl $crate::atomic::Raw<$loose> for $atomic {
            #[inline]
            fn new_(loose: $loose) -> Self {
                <$atomic>::new(loose)
            }

            #[inline]
            fn load_(&self, ordering: ::core::sync::atomic::Ordering) -> $loose {
                self.load(ordering)
            }

            #[inline]
            fn store_(&self, value: $loose, ordering: ::core::sync::atomic::Ordering) {
                self.store(value, ordering)
            }

            #[inline]
            fn get_(&mut self) -> $loose {
                *self.get_mut()
            }

            #[inline]
            fn set_(&mut self, value: $loose) {
                *self.get_mut() = value
            }

            #[inline]
            fn compare_exchange_(
                &self,
                old: $loose,
                new: $loose,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$loose, $loose> {
                self.compare_exchange(old, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak_(
                &self,
                old: $loose,
                new: $loose,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$loose, $loose> {
                self.compare_exchange_weak(old, new, success, failure)
            }

            #[inline]
            fn swap_(&self, value: $loose, ordering: ::core::sync::atomic::Ordering) -> $loose {
                self.swap(value, ordering)
            }
        }
    };
}

impl_raw!(u8, AtomicU8);
impl_raw!(u16, AtomicU16);
impl_raw!(u32, AtomicU32);
impl_raw!(u64, AtomicU64);
#[cfg(feature = "u128")]
impl_raw!(u128, AtomicU128);
