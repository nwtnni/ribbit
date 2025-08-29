use std::fmt::Display;

use crate::ty;

#[derive(Debug)]
pub enum Error {
    Overflow {
        /// Which bit the field should start at.
        offset: usize,

        /// How many bits the underlying representation has available at `offset`.
        available: usize,

        /// How many bits the field requires.
        required: usize,
    },

    TopLevelSize,
    StructNonZero,
    OpaqueSize,
    WrongSize {
        ty: ty::Tight,
        expected: usize,
        actual: usize,
    },
    ArbitraryNonZero,
    ArbitrarySize {
        size: usize,
    },
    UnsupportedType,
    VariantSize {
        variant: usize,
        r#enum: usize,
        discriminant: usize,
    },
}

macro_rules! bail {
    ($span:expr => $error:expr) => {{
        #[allow(unused_imports)]
        use ::syn::spanned::Spanned as _;
        return Err(darling::Error::custom($error).with_span(&$span.span()));
    }};
}

pub(crate) use bail;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Overflow {
                offset,
                available,
                required,
            } => {
                write!(f, "Field requires {required} bits at offset {offset}, but only {available} are available")
            }
            Error::StructNonZero => {
                write!(
                    f,
                    "At least one field must be non-zero for struct to be non-zero",
                )
            }
            Error::OpaqueSize => {
                write!(
                    f,
                    "Opaque type requires size attribute #[ribbit(size = ...)]"
                )
            }
            Error::WrongSize {
                ty,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Size attribute #[ribbit(size = {expected})] does not match size of {ty}: {actual}",
                )
            }
            Error::ArbitraryNonZero => {
                write!(f, "Nonzero arbitrary sizes are currently unsupported",)
            }
            Error::ArbitrarySize { size } => {
                write!(f, "Arbitrary size {size} unsupported")
            }
            Error::UnsupportedType => {
                write!(f, "Only type paths are supported")
            }
            Error::TopLevelSize => {
                write!(f, "#[ribbit(size = ...)] is required at the top level")
            }
            Error::VariantSize {
                variant,
                r#enum,
                discriminant,
            } => {
                write!(
                    f,
                    "Variant of size {variant} does not fit in enum of size {enum} with discriminant of size {discriminant}",
                )
            }
        }
    }
}
