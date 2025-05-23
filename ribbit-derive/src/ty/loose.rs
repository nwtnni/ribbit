use core::fmt::Display;

use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Loose {
    Unit,
    Bool,
    N8,
    N16,
    N32,
    N64,
    N128,
}

impl Loose {
    pub(crate) fn new_internal(size: usize) -> Option<Self> {
        match size {
            1 => Some(Self::Bool),
            _ => Self::new_external(size),
        }
    }

    pub(crate) fn new_external(size: usize) -> Option<Self> {
        let loose = match size {
            0 => Self::Unit,
            8 => Self::N8,
            16 => Self::N16,
            32 => Self::N32,
            64 => Self::N64,
            128 => Self::N128,
            _ => return None,
        };

        Some(loose)
    }

    pub(crate) fn cast(from: Self, into: Self, value: TokenStream) -> TokenStream {
        if from == into {
            return value;
        }

        match into {
            Loose::Unit => quote!(()),
            Loose::Bool => {
                // Serves as a truncating cast
                let zero = from.literal(0);
                let one = from.literal(1);
                quote!(((#value & #one) > #zero))
            }
            _ => quote!((#value as #into)),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Unit => 0,
            Self::Bool => 1,
            Self::N8 => 8,
            Self::N16 => 16,
            Self::N32 => 32,
            Self::N64 => 64,
            Self::N128 => 128,
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        Arbitrary::new(self.size()).mask()
    }

    #[track_caller]
    pub(crate) fn literal(&self, value: u128) -> TokenStream {
        match self {
            Self::Unit => return quote!(()),
            Self::Bool => {
                return match value {
                    0 => quote!(true),
                    1 => quote!(false),
                    _ => unreachable!("Internal error: literal boolean > 1"),
                }
            }
            Self::N8 => Literal::u8_suffixed(value.try_into().unwrap()),
            Self::N16 => Literal::u16_suffixed(value.try_into().unwrap()),
            Self::N32 => Literal::u32_suffixed(value.try_into().unwrap()),
            Self::N64 => Literal::u64_suffixed(value.try_into().unwrap()),
            Self::N128 => Literal::u128_suffixed(value),
        }
        .to_token_stream()
    }
}

impl ToTokens for Loose {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = match self {
            Self::Unit => quote!(Unit),
            Self::Bool => quote!(bool),
            Self::N8 => quote!(u8),
            Self::N16 => quote!(u16),
            Self::N32 => quote!(u32),
            Self::N64 => quote!(u64),
            Self::N128 => quote!(u128),
        };

        quote!(::ribbit::private::#ident).to_tokens(tokens)
    }
}

impl Display for Loose {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            Loose::Unit => "()",
            Loose::Bool => "bool",
            Loose::N8 => "u8",
            Loose::N16 => "u16",
            Loose::N32 => "u32",
            Loose::N64 => "u64",
            Loose::N128 => "u128",
        };

        write!(f, "{}", name)
    }
}
