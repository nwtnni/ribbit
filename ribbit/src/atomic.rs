use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use crate::convert::loose_to_packed;
use crate::convert::packed_to_loose;
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

/// Type-safe atomic wrapper for unpacked type implementing [`Pack`].
///
/// Generic type parameter `R` defaults to standard library and `portable_atomic`
/// atomic integer types, but can be overridden.
#[repr(transparent)]
pub struct Atomic<U, R = <<<U as Pack>::Packed as Unpack>::Loose as Loose>::Atomic> {
    raw: R,
    unpacked: PhantomData<U>,
}

impl<U, R> Atomic<U, R> {
    /// `const` constructor.
    #[inline]
    pub const fn from_raw(raw: R) -> Self {
        Self {
            raw,
            unpacked: PhantomData,
        }
    }

    // https://github.com/rust-lang/rust/issues/73255
    // #[inline]
    // pub const fn into_raw(self) -> R {
    //     self.raw
    // }
}

impl<U, R> Atomic<U, R>
where
    U: Pack,
    R: Raw<<<U as Pack>::Packed as Unpack>::Loose>,
{
    /// Equivalent to [`core::sync::atomic::AtomicU64::new`].
    #[inline]
    pub fn new(unpacked: U) -> Self {
        Self::new_packed(unpacked.pack())
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::new`], but takes a packed value.
    #[inline]
    pub fn new_packed(packed: U::Packed) -> Self {
        Self::from_raw(R::new_(packed_to_loose(packed)))
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::load`].
    #[inline]
    pub fn load(&self, ordering: Ordering) -> U {
        self.load_packed(ordering).unpack()
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::load`], but does not unpack.
    #[inline]
    pub fn load_packed(&self, ordering: Ordering) -> U::Packed {
        let raw = R::load_(&self.raw, ordering);
        // SAFETY: API inductively preserves packed type invariants
        unsafe { loose_to_packed(raw) }
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::store`].
    #[inline]
    pub fn store(&self, value: U, ordering: Ordering) {
        self.store_packed(value.pack(), ordering)
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::store`], but takes a packed value.
    #[inline]
    pub fn store_packed(&self, value: U::Packed, ordering: Ordering) {
        R::store_(&self.raw, packed_to_loose(value), ordering)
    }

    /// Like [`core::sync::atomic::AtomicU64::get_mut`], but returns a copy.
    ///
    /// This is necessary because the unpacked type usually has a different memory layout.
    /// Also see [`Atomic::set`] and [`Atomic::get_mut_packed`].
    #[inline]
    pub fn get(&mut self) -> U {
        self.get_mut_packed().unpack()
    }

    /// Like [`core::sync::atomic::AtomicU64::get_mut`], but stores a copy.
    ///
    /// This is necessary because the unpacked type usually has a different memory layout.
    /// Also see [`Atomic::get`] and [`Atomic::get_mut_packed`].
    #[inline]
    pub fn set(&mut self, value: U) {
        *self.get_mut_packed() = value.pack();
    }

    /// Like [`core::sync::atomic::AtomicU64::get_mut`], but does not unpack.
    #[inline]
    pub fn get_mut_packed(&mut self) -> &mut U::Packed {
        const {
            assert!(
                core::mem::size_of::<<crate::Packed<U> as Unpack>::Loose>()
                    == core::mem::size_of::<U::Packed>()
            );

            assert!(
                core::mem::align_of::<<crate::Packed<U> as Unpack>::Loose>()
                    == core::mem::align_of::<U::Packed>()
            );
        }

        // SAFETY: checked above that referenced types have same layout
        unsafe {
            core::mem::transmute::<&mut <<U as Pack>::Packed as Unpack>::Loose, &mut U::Packed>(
                R::get_mut_(&mut self.raw),
            )
        }
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::compare_exchange`].
    #[inline]
    pub fn compare_exchange(
        &self,
        old: U,
        new: U,
        success: Ordering,
        failure: Ordering,
    ) -> Result<U, U> {
        self.compare_exchange_packed(old.pack(), new.pack(), success, failure)
            .map(Unpack::unpack)
            .map_err(Unpack::unpack)
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::compare_exchange`], but takes packed
    /// values and does not unpack.
    #[inline]
    pub fn compare_exchange_packed(
        &self,
        old: U::Packed,
        new: U::Packed,
        success: Ordering,
        failure: Ordering,
    ) -> Result<U::Packed, U::Packed> {
        R::compare_exchange_(
            &self.raw,
            packed_to_loose(old),
            packed_to_loose(new),
            success,
            failure,
        )
        // SAFETY: API inductively preserves packed type invariants
        .map(|old| unsafe { loose_to_packed(old) })
        .map_err(|old| unsafe { loose_to_packed(old) })
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::compare_exchange_weak`].
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        old: U,
        new: U,
        success: Ordering,
        failure: Ordering,
    ) -> Result<U, U> {
        self.compare_exchange_weak_packed(old.pack(), new.pack(), success, failure)
            .map(Unpack::unpack)
            .map_err(Unpack::unpack)
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::compare_exchange_weak`], but takes packed
    /// values and does not unpack.
    #[inline]
    pub fn compare_exchange_weak_packed(
        &self,
        old: U::Packed,
        new: U::Packed,
        success: Ordering,
        failure: Ordering,
    ) -> Result<U::Packed, U::Packed> {
        R::compare_exchange_weak_(
            &self.raw,
            packed_to_loose(old),
            packed_to_loose(new),
            success,
            failure,
        )
        // SAFETY: API inductively preserves packed type invariants
        .map(|old| unsafe { loose_to_packed(old) })
        .map_err(|old| unsafe { loose_to_packed(old) })
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::swap`].
    #[inline]
    pub fn swap(&self, value: U, ordering: Ordering) -> U {
        self.swap_packed(value.pack(), ordering).unpack()
    }

    /// Equivalent to [`core::sync::atomic::AtomicU64::swap`], but takes a packed
    /// value and does not unpack.
    #[inline]
    pub fn swap_packed(&self, value: U::Packed, ordering: Ordering) -> U::Packed {
        let raw = R::swap_(&self.raw, packed_to_loose(value), ordering);
        // SAFETY: API inductively preserves packed type invariants
        unsafe { loose_to_packed(raw) }
    }
}

impl<U, R> Clone for Atomic<U, R>
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

impl<U, R> Debug for Atomic<U, R>
where
    U: Pack,
    U::Packed: Debug,
    R: Raw<<<U as Pack>::Packed as Unpack>::Loose>,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<U, R> Display for Atomic<U, R>
where
    U: Pack,
    U::Packed: Display,
    R: Raw<<<U as Pack>::Packed as Unpack>::Loose>,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.load_packed(Ordering::Relaxed).fmt(f)
    }
}

impl<U, R> Default for Atomic<U, R>
where
    R: Default,
{
    #[inline]
    fn default() -> Self {
        Self::from_raw(R::default())
    }
}

impl<U, R> From<R> for Atomic<U, R> {
    #[inline]
    fn from(raw: R) -> Self {
        Self::from_raw(raw)
    }
}

/// Interface for underlying atomic integer.
pub trait Raw<T>: core::fmt::Debug + Default + Send + Sync {
    fn new_(value: T) -> Self;

    fn load_(&self, ordering: Ordering) -> T;

    fn store_(&self, value: T, ordering: Ordering);

    fn get_mut_(&mut self) -> &mut T;

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
///
/// Should be called like: `impl_raw!(u64, my_atomic::AtomicU64)`.
/// Forwards by calling the corresponding methods (with no trailing underscore)
/// on the second type, which allows for some basic duck typing
/// (e.g., second type can be a wrapper implementing [`core::ops::Deref`]).
#[macro_export]
macro_rules! impl_raw {
    ($raw:ty, $atomic:ty) => {
        impl $crate::atomic::Raw<$raw> for $atomic {
            #[inline]
            fn new_(value: $raw) -> Self {
                <$atomic>::new(value)
            }

            #[inline]
            fn load_(&self, ordering: ::core::sync::atomic::Ordering) -> $raw {
                self.load(ordering)
            }

            #[inline]
            fn store_(&self, value: $raw, ordering: ::core::sync::atomic::Ordering) {
                self.store(value, ordering)
            }

            #[inline]
            fn get_mut_(&mut self) -> &mut $raw {
                self.get_mut()
            }

            #[inline]
            fn compare_exchange_(
                &self,
                old: $raw,
                new: $raw,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$raw, $raw> {
                self.compare_exchange(old, new, success, failure)
            }

            #[inline]
            fn compare_exchange_weak_(
                &self,
                old: $raw,
                new: $raw,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$raw, $raw> {
                self.compare_exchange_weak(old, new, success, failure)
            }

            #[inline]
            fn swap_(&self, value: $raw, ordering: ::core::sync::atomic::Ordering) -> $raw {
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
