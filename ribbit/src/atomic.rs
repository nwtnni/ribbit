use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use crate::Pack;
use crate::Unpack;

pub use core::sync::atomic::AtomicU16;
pub use core::sync::atomic::AtomicU32;
pub use core::sync::atomic::AtomicU64;
pub use core::sync::atomic::AtomicU8;
#[cfg(feature = "atomic-u128")]
pub use portable_atomic::AtomicU128;

#[repr(transparent)]
pub struct Atomic<T: Pack, R = <<<T as Pack>::Packed as Unpack>::Loose as Loose>::Atomic> {
    raw: R,
    r#type: PhantomData<T>,
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
        Self::new_raw(R::new(Self::packed_to_loose(packed)))
    }

    #[inline]
    pub const fn new_raw(raw: R) -> Self {
        Self {
            raw,
            r#type: PhantomData,
        }
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> T {
        self.load_packed(ordering).unpack()
    }

    #[inline]
    pub fn load_packed(&self, ordering: Ordering) -> T::Packed {
        Self::loose_to_packed(R::load(&self.raw, ordering))
    }

    #[inline]
    pub fn store(&self, value: T, ordering: Ordering) {
        self.store_packed(value.pack(), ordering)
    }

    #[inline]
    pub fn store_packed(&self, value: T::Packed, ordering: Ordering) {
        R::store(&self.raw, Self::packed_to_loose(value), ordering)
    }

    #[inline]
    pub fn get(&mut self) -> T {
        self.get_packed().unpack()
    }

    #[inline]
    pub fn get_packed(&mut self) -> T::Packed {
        Self::loose_to_packed(R::get(&mut self.raw))
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.set_packed(value.pack())
    }

    #[inline]
    pub fn set_packed(&mut self, value: T::Packed) {
        R::set(&mut self.raw, Self::packed_to_loose(value))
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
        R::compare_exchange(
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
        R::compare_exchange_weak(
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
        Self::loose_to_packed(R::swap(&self.raw, Self::packed_to_loose(value), ordering))
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
    <T::Packed as Unpack>::Loose: Loose,
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
    <T::Packed as Unpack>::Loose: Loose,
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
    <T::Packed as Unpack>::Loose: Loose,
{
    #[inline]
    fn default() -> Self {
        Self::new_packed(T::Packed::default())
    }
}

/// Interface for underlying atomic library.
pub trait Raw<T>: core::fmt::Debug + Default + Send + Sync {
    fn new(loose: T) -> Self;

    fn load(&self, ordering: Ordering) -> T;

    fn store(&self, value: T, ordering: Ordering);

    fn get(&mut self) -> T;

    fn set(&mut self, value: T);

    fn compare_exchange(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    fn compare_exchange_weak(
        &self,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    fn swap(&self, value: T, ordering: Ordering) -> T;
}

#[macro_export]
macro_rules! impl_raw {
    ($loose:ty, $atomic:ty) => {
        impl $crate::atomic::Raw<$loose> for $atomic {
            #[inline]
            fn new(loose: $loose) -> Self {
                <$atomic>::new(loose)
            }

            #[inline]
            fn load(&self, ordering: Ordering) -> $loose {
                <$atomic>::load(self, ordering)
            }

            #[inline]
            fn store(&self, value: $loose, ordering: Ordering) {
                <$atomic>::store(self, value, ordering)
            }

            #[inline]
            fn get(&mut self) -> $loose {
                *<$atomic>::get_mut(self)
            }

            #[inline]
            fn set(&mut self, value: $loose) {
                *<$atomic>::get_mut(self) = value
            }

            #[inline]
            fn compare_exchange(
                &self,
                old: $loose,
                new: $loose,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$loose, $loose> {
                <$atomic>::compare_exchange(self, old, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak(
                &self,
                old: $loose,
                new: $loose,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$loose, $loose> {
                <$atomic>::compare_exchange_weak(self, old, new, success, failure)
            }

            #[inline]
            fn swap(&self, value: $loose, ordering: Ordering) -> $loose {
                <$atomic>::swap(self, value, ordering)
            }
        }
    };
}

/// Provides default implementations of [`Raw`].
#[expect(private_bounds)]
pub trait Loose: crate::Loose {
    type Atomic: Raw<Self>;
}

impl_raw!(u8, AtomicU8);
impl Loose for u8 {
    type Atomic = AtomicU8;
}

impl_raw!(u16, AtomicU16);
impl Loose for u16 {
    type Atomic = AtomicU16;
}

impl_raw!(u32, AtomicU32);
impl Loose for u32 {
    type Atomic = AtomicU32;
}

impl_raw!(u64, AtomicU64);
impl Loose for u64 {
    type Atomic = AtomicU64;
}

#[cfg(feature = "atomic-u128")]
impl_raw!(u128, AtomicU128);
#[cfg(feature = "atomic-u128")]
impl Loose for u128 {
    type Atomic = AtomicU128;
}
