use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::repr::Arbitrary;

#[derive(Copy, Clone, PartialEq, Eq)]
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

    pub(crate) fn to_native<T: ToTokens>(&self, input: T) -> TokenStream {
        quote!(#input as #self)
    }
}

impl ToTokens for Native {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = match self {
            Native::N8 => quote!(u8),
            Native::N16 => quote!(u16),
            Native::N32 => quote!(u32),
            Native::N64 => quote!(u64),
        };

        quote!(::ribbit::private::#name).to_tokens(tokens)
    }
}
