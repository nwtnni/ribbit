use proc_macro2::Literal;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Loose {
    N8,
    N16,
    N32,
    N64,
}

impl Loose {
    pub(crate) fn size(&self) -> usize {
        match self {
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
    pub(crate) fn literal(&self, value: usize) -> Literal {
        match self {
            Loose::N8 => Literal::u8_suffixed(value.try_into().unwrap()),
            Loose::N16 => Literal::u16_suffixed(value.try_into().unwrap()),
            Loose::N32 => Literal::u32_suffixed(value.try_into().unwrap()),
            Loose::N64 => Literal::u64_suffixed(value.try_into().unwrap()),
        }
    }
}

impl ToTokens for Loose {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = match self {
            Loose::N8 => quote!(u8),
            Loose::N16 => quote!(u16),
            Loose::N32 => quote!(u32),
            Loose::N64 => quote!(u64),
        };

        quote!(::ribbit::private::#ident).to_tokens(tokens)
    }
}
