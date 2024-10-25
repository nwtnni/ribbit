use proc_macro2::Literal;
use quote::quote;
use quote::ToTokens;

use crate::repr::Arbitrary;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Native {
    N8,
    N16,
    N32,
    N64,
}

impl Native {
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
            Native::N8 => Literal::u8_suffixed(value.try_into().unwrap()),
            Native::N16 => Literal::u16_suffixed(value.try_into().unwrap()),
            Native::N32 => Literal::u32_suffixed(value.try_into().unwrap()),
            Native::N64 => Literal::u64_suffixed(value.try_into().unwrap()),
        }
    }
}

impl ToTokens for Native {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = match self {
            Native::N8 => quote!(u8),
            Native::N16 => quote!(u16),
            Native::N32 => quote!(u32),
            Native::N64 => quote!(u64),
        };

        quote!(::ribbit::private::#ident).to_tokens(tokens)
    }
}
