use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::TypePath;

use crate::ty::Tight;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    path: TypePath,
    tight: Tight,
}

impl Node {
    pub(crate) fn parse(path: TypePath, tight: Tight) -> Self {
        Self { path, tight }
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}

impl Deref for Node {
    type Target = Tight;
    fn deref(&self) -> &Self::Target {
        &self.tight
    }
}
