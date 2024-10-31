use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::TypePath;

use crate::ty::Tight;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    path: TypePath,
    repr: Tight,
}

impl Node {
    pub(crate) fn from_path(path: TypePath, repr: Tight) -> Self {
        Self { path, repr }
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
        &self.repr
    }
}
