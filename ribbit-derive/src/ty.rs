mod arbitrary;
mod loose;
mod node;
pub(crate) mod tight;

pub(crate) use arbitrary::Arbitrary;
pub(crate) use loose::Loose;
pub(crate) use node::Node;
pub(crate) use tight::Tight;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::error::bail;
use crate::Error;
use crate::Spanned;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Node(Node),
    Leaf(Tight),
}

impl Tree {
    pub(crate) fn from_ty(
        ty: syn::Type,
        nonzero: Option<Spanned<bool>>,
        size: Option<Spanned<usize>>,
    ) -> darling::Result<Spanned<Self>> {
        match ty {
            syn::Type::Path(path) => {
                let span = path.span();

                let repr = match Tight::from_path(&path) {
                    Some(leaf) => Self::Leaf(leaf),
                    None => {
                        let Some(size) = size else {
                            bail!(span=> Error::OpaqueSize);
                        };

                        let leaf = Tight::new(nonzero.unwrap_or_else(|| false.into()), size);

                        Self::Node(Node::from_path(path, leaf))
                    }
                };

                Ok(Spanned::new(repr, span))
            }
            _ => bail!(ty=> Error::UnsupportedType),
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self, Tree::Leaf(_))
    }

    pub(crate) fn to_leaf(&self) -> Tight {
        match self {
            Tree::Node(node) => **node,
            Tree::Leaf(leaf) => *leaf,
        }
    }

    pub(crate) fn to_native(&self) -> Loose {
        self.to_leaf().to_native()
    }

    pub(crate) fn size(&self) -> Spanned<usize> {
        match self {
            Tree::Node(node) => node.size(),
            Tree::Leaf(leaf) => leaf.size(),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Tree::Node(node) => node.mask(),
            Tree::Leaf(leaf) => leaf.mask(),
        }
    }

    pub(crate) fn nonzero(&self) -> Spanned<bool> {
        match self {
            Tree::Node(node) => node.nonzero,
            Tree::Leaf(leaf) => leaf.nonzero,
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
    fn from(native: Loose) -> Self {
        Self::Leaf(Tight::from(native))
    }
}
