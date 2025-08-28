mod arbitrary;
mod loose;
mod node;
pub(crate) mod tight;

pub(crate) use arbitrary::Arbitrary;
use darling::usage::IdentSet;
pub(crate) use loose::Loose;
pub(crate) use node::Node;
pub(crate) use tight::Tight;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::error::bail;
use crate::Error;
use crate::Spanned;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tree {
    Node(Node),
    Leaf(Tight),
}

impl Tree {
    pub(crate) fn parse(
        ty_params: &IdentSet,
        ty: syn::Type,
        nonzero: Option<Spanned<bool>>,
        size: Option<Spanned<usize>>,
    ) -> darling::Result<Spanned<Self>> {
        match ty {
            syn::Type::Path(path) => {
                let span = path.span();

                let ty = match Tight::from_path(&path) {
                    Some(tight) => {
                        let tight = match tight {
                            Ok(tight) => tight,
                            Err(error) => bail!(span=> error),
                        };

                        if let Some(expected) = size.filter(|size| **size != tight.size()) {
                            bail!(span=> Error::WrongSize {
                                expected: *expected,
                                actual: tight.size(),
                                ty: tight,
                            });
                        }

                        Self::Leaf(tight)
                    }
                    None => {
                        let Some(size) = size else {
                            bail!(span=> Error::OpaqueSize);
                        };

                        let tight =
                            Tight::from_size(nonzero.as_deref().copied().unwrap_or(false), *size);

                        let tight = match tight {
                            Ok(tight) => tight,
                            Err(error) => bail!(span=> error),
                        };

                        Self::Node(Node::parse(ty_params, path, tight))
                    }
                };

                Ok(Spanned::new(ty, span))
            }
            _ => bail!(ty=> Error::UnsupportedType),
        }
    }

    pub(crate) fn is_node(&self) -> bool {
        matches!(self, Tree::Node(_))
    }

    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self, Tree::Leaf(_))
    }

    pub(crate) fn packed(&self) -> TokenStream {
        match self {
            Tree::Leaf(_) => quote!(#self),
            Tree::Node(node) => quote!(<#node as ::ribbit::Pack>::Packed),
        }
    }

    pub(crate) fn pack(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tree::Leaf(_) => expression,
            Tree::Node(_) => quote!(#expression.pack()),
        }
    }

    pub(crate) fn unpack(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tree::Leaf(_) => expression,
            Tree::Node(_) => quote!(#expression.unpack()),
        }
    }

    pub(crate) fn tighten(&self) -> Tight {
        match self {
            Tree::Node(node) => node.tighten().clone(),
            Tree::Leaf(leaf) => leaf.clone(),
        }
    }

    pub(crate) fn loosen(&self) -> &Loose {
        match self {
            Tree::Node(node) => node.loosen(),
            Tree::Leaf(leaf) => leaf.loosen(),
        }
    }

    pub(crate) fn size_expected(&self) -> usize {
        match self {
            Tree::Node(node) => node.tighten().size(),
            Tree::Leaf(leaf) => leaf.size(),
        }
    }

    pub(crate) fn size_actual(&self) -> TokenStream {
        quote!(<#self as ::ribbit::Pack>::BITS)
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Tree::Node(node) => node.tighten().mask(),
            Tree::Leaf(leaf) => leaf.mask(),
        }
    }

    pub(crate) fn is_nonzero(&self) -> bool {
        match self {
            Tree::Node(node) => node.tighten().is_nonzero(),
            Tree::Leaf(leaf) => leaf.is_nonzero(),
        }
    }
}

impl ToTokens for Tree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Tree::Node(node) => node.to_tokens(tokens),
            Tree::Leaf(leaf) => leaf.to_tokens(tokens),
        }
    }
}

impl From<Tight> for Tree {
    fn from(leaf: Tight) -> Self {
        Self::Leaf(leaf)
    }
}

impl From<Loose> for Tree {
    fn from(loose: Loose) -> Self {
        Self::Leaf(Tight::from(loose))
    }
}

fn mask(size: usize) -> u128 {
    assert!(
        size <= 128,
        "[INTERNAL ERROR]: cannot mask size > 128: {size}",
    );

    1u128
        .checked_shl(size as u32)
        .and_then(|mask| mask.checked_sub(1))
        .unwrap_or(u128::MAX)
}
