use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::ty::Loose;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Arbitrary {
    size: usize,
}

impl Arbitrary {
    pub(super) fn new(size: usize) -> Self {
        Self { size }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn mask(&self) -> usize {
        1usize
            .checked_shl(self.size as u32)
            .and_then(|mask| mask.checked_sub(1))
            .unwrap_or(usize::MAX)
    }

    pub(crate) fn loosen(&self) -> Loose {
        match self.size {
            0..=7 => Loose::N8,
            9..=15 => Loose::N16,
            17..=31 => Loose::N32,
            33..=63 => Loose::N64,
            _ => unreachable!(),
        }
    }
}

impl ToTokens for Arbitrary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = format_ident!("u{}", self.size());
        quote!(::ribbit::private::#ident).to_tokens(tokens)
    }
}

impl Display for Arbitrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "u{}", self.size())
    }
}
