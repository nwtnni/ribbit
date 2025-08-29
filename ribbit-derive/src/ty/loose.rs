use core::fmt::Display;
use core::str::FromStr as _;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Loose {
    N8,
    N16,
    N32,
    N64,
    N128,
}

impl Loose {
    pub(crate) fn new(size: usize) -> Option<Self> {
        let loose = match size {
            8 => Self::N8,
            16 => Self::N16,
            32 => Self::N32,
            64 => Self::N64,
            128 => Self::N128,
            _ => return None,
        };

        Some(loose)
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Self::N8 => 8,
            Self::N16 => 16,
            Self::N32 => 32,
            Self::N64 => 64,
            Self::N128 => 128,
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        crate::mask(self.size())
    }

    #[track_caller]
    pub(crate) fn literal(&self, value: u128) -> TokenStream {
        TokenStream::from_str(&format!("{value:#X}{self}")).unwrap()
    }
}

impl ToTokens for Loose {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = match self {
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
            Loose::N8 => "u8",
            Loose::N16 => "u16",
            Loose::N32 => "u32",
            Loose::N64 => "u64",
            Loose::N128 => "u128",
        };

        write!(f, "{name}")
    }
}
