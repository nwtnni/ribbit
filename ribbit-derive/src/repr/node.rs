use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::TypePath;

use crate::repr::Leaf;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node<'input> {
    path: &'input TypePath,
    repr: Leaf,
}

impl<'input> Node<'input> {
    pub(crate) fn from_path(path: &'input TypePath, repr: Leaf) -> Self {
        Self { path, repr }
    }
}

impl ToTokens for Node<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}

impl Deref for Node<'_> {
    type Target = Leaf;
    fn deref(&self) -> &Self::Target {
        &self.repr
    }
}
