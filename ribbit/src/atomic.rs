use core::fmt::Debug;
use core::fmt::Display;
use core::mem::ManuallyDrop;
use core::sync::atomic::Ordering;

use crate::Pack;
use crate::Unpack;

#[repr(transparent)]
pub struct Atomic<T: Pack>(<<T::Packed as Unpack>::Loose as Loose>::Atomic)
where
    <T::Packed as Unpack>::Loose: Loose;

impl<T: Pack> Atomic<T>
where
    <T::Packed as Unpack>::Loose: Loose,
{
    #[inline]
    pub fn new(value: T) -> Self {
        Self(<<T::Packed as Unpack>::Loose as Loose>::new(
            Self::packed_to_loose(value.pack()),
        ))
    }

    #[inline]
    pub const fn new_packed(value: T::Packed) -> Self {
        union Transmute<T: Unpack>
        where
            T::Loose: Loose,
        {
            loose: T::Loose,
            atomic: ManuallyDrop<<T::Loose as Loose>::Atomic>,
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
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as Loose>::load(
            &self.0, ordering,
        ))
    }

    #[inline]
    pub fn store(&self, value: T, ordering: Ordering) {
        self.store_packed(value.pack(), ordering)
    }

    #[inline]
    pub fn store_packed(&self, value: T::Packed, ordering: Ordering) {
        <<T::Packed as Unpack>::Loose as Loose>::store(
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
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as Loose>::get(&mut self.0))
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.set_packed(value.pack())
    }

    #[inline]
    pub fn set_packed(&mut self, value: T::Packed) {
        <<T::Packed as Unpack>::Loose as Loose>::set(&mut self.0, Self::packed_to_loose(value))
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
        <<T::Packed as Unpack>::Loose as Loose>::compare_exchange(
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
        <<T::Packed as Unpack>::Loose as Loose>::compare_exchange_weak(
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
        Self::loose_to_packed(<<T::Packed as Unpack>::Loose as Loose>::swap(
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

macro_rules! impl_atomic {
    ($ty:ty, $atomic:ty) => {
        impl Loose for $ty {
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

#[expect(private_bounds)]
pub trait Loose: crate::Loose {
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

#[cfg(feature = "atomic-u128")]
impl_atomic!(u128, portable_atomic::AtomicU128);
