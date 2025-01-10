use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Loose {
    Unit,
    N8,
    N16,
    N32,
    N64,
}

impl Loose {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Unit => 0,
            Self::N8 => 8,
            Self::N16 => 16,
            Self::N32 => 32,
            Self::N64 => 64,
        }
    }

    pub(crate) fn mask(&self) -> usize {
        Arbitrary::new(self.size()).mask()
    }

    #[track_caller]
    pub(crate) fn literal(&self, value: usize) -> TokenStream {
        match self {
            Self::Unit => return quote!(()),
            Self::N8 => Literal::u8_suffixed(value.try_into().unwrap()),
            Self::N16 => Literal::u16_suffixed(value.try_into().unwrap()),
            Self::N32 => Literal::u32_suffixed(value.try_into().unwrap()),
            Self::N64 => Literal::u64_suffixed(value.try_into().unwrap()),
        }
        .to_token_stream()
    }
}

impl ToTokens for Loose {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = match self {
            Self::Unit => quote!(Unit),
            Self::N8 => quote!(u8),
            Self::N16 => quote!(u16),
            Self::N32 => quote!(u32),
            Self::N64 => quote!(u64),
        };

        quote!(::ribbit::private::#ident).to_tokens(tokens)
    }
}
