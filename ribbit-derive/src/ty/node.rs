use darling::usage::CollectTypeParams;
use darling::usage::IdentSet;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::TypePath;

use crate::ty::Loose;
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

    pub(crate) fn tighten(&self) -> &Tight {
        &self.tight
    }

    pub(crate) fn loosen(&self) -> Loose {
        self.tight.loosen()
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}
