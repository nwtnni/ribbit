use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;
use syn::TypeParam;
use syn::TypePath;

use crate::ty::Leaf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    path: TypePath,
    repr: Leaf,
}

impl Node {
    pub(crate) fn from_path(path: TypePath, repr: Leaf) -> Self {
        Self { path, repr }
    }

    pub(crate) fn contains(&self, param: &TypeParam) -> bool {
        fn recurse(path: &TypePath, param: &TypeParam) -> bool {
            path.path.segments.iter().any(|segment| {
                segment.ident == param.ident
                    || match &segment.arguments {
                        PathArguments::None | PathArguments::Parenthesized(_) => false,
                        PathArguments::AngleBracketed(args) => {
                            args.args.iter().any(|arg| match arg {
                                GenericArgument::Type(Type::Path(path)) => recurse(path, param),
                                _ => todo!(),
                            })
                        }
                    }
            })
        }

        recurse(&self.path, param)
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}

impl Deref for Node {
    type Target = Leaf;
    fn deref(&self) -> &Self::Target {
        &self.repr
    }
}
