use std::fmt::Display;

pub enum Error {
    Overflow {
        /// Which bit the field should start at.
        offset: usize,

        /// How many bits the underlying representation has available at `offset`.
        available: usize,

        /// How many bits the field requires.
        required: usize,
    },

    Underflow {
        bits: BitBox,
    },

    StructNonZero,
    OpaqueSize,
    ArbitraryNonZero,
    UnsupportedType,
}

macro_rules! bail {
    ($span:expr => $error:expr) => {{
        #[allow(unused_imports)]
        use ::syn::spanned::Spanned as _;
        return Err(darling::Error::custom($error).with_span(&$span.span()));
    }};
}

pub(crate) use bail;
use bitvec::boxed::BitBox;

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
            Error::Underflow { bits } => {
                write!(f, "All bits must be used: {bits:?}")
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
            Error::ArbitraryNonZero => {
                write!(f, "Nonzero arbitrary sizes are currently unsupported",)
            }
            Error::UnsupportedType => {
                write!(f, "Only type paths are supported")
            }
        }
    }
}
