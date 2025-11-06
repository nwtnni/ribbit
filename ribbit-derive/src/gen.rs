// Types
pub(crate) mod packed;
pub(crate) use packed::packed;

// Methods
pub(crate) mod get;
pub(crate) use get::get;

pub(crate) mod new;
pub(crate) use new::new;

pub(crate) mod precondition;
pub(crate) use precondition::precondition;

pub(crate) mod with;
pub(crate) use with::with;

// Traits

/// Generate a [`core::fmt::Debug`] implementation by forwarding to the unpacked type.
pub(crate) mod debug;
pub(crate) use debug::debug;

/// Generate [`core::cmp::PartialEq`] and [`core::cmp::Eq`] implementations
/// for the packed type directly based on the underlying tight type.
pub(crate) mod eq;
pub(crate) use eq::eq;

/// Generate [`From`] implementations between the packed and unpacked types.
pub(crate) mod from;
pub(crate) use from::from;

/// Generate a [`core::hash::Hash`] implementation for the packed type
/// directly based on the underlying tight type.
pub(crate) mod hash;
pub(crate) use hash::hash;

mod nonzero;
pub(crate) use nonzero::nonzero;

/// Generate [`core::cmp::PartialOrd`] and [`core::cmp::Ord`] implementations
/// for the packed type directly based on the underlying tight type.
pub(crate) mod ord;
pub(crate) use ord::ord;

mod pack;
pub(crate) use pack::pack;

mod unpack;
pub(crate) use unpack::unpack;
