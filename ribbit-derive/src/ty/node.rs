use core::ops::Deref;

use darling::usage::CollectTypeParams;
use darling::usage::IdentSet;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::TypePath;

use crate::ty::Tight;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    path: TypePath,
    uses: IdentSet,
    tight: Tight,
}

impl Node {
    pub(crate) fn parse(ty_params: &IdentSet, path: TypePath, tight: Tight) -> Self {
        let uses = std::iter::once(&path)
            .collect_type_params_cloned(&darling::usage::Purpose::Declare.into(), ty_params);

        Self { path, uses, tight }
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
