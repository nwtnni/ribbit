mod arbitrary;
pub(crate) mod leaf;
mod native;
mod node;

pub(crate) use arbitrary::Arbitrary;
pub(crate) use leaf::Leaf;
pub(crate) use native::Native;
pub(crate) use node::Node;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::error::bail;
use crate::Error;
use crate::Spanned;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tree<'input> {
    Node(Node<'input>),
    Leaf(Leaf),
}

impl<'input> Tree<'input> {
    pub(crate) fn from_ty(
        ty: &'input syn::Type,
        nonzero: Option<Spanned<bool>>,
        size: Option<Spanned<usize>>,
    ) -> darling::Result<Spanned<Self>> {
        match ty {
            syn::Type::Path(path) => {
                let span = path.span();

                let repr = match Leaf::from_path(path) {
                    Some(leaf) => Self::Leaf(leaf),
                    None => {
                        let Some(size) = size else {
                            bail!(ty=> Error::OpaqueSize);
                        };

                        let leaf = Leaf::new(nonzero.unwrap_or_else(|| false.into()), size);

                        Self::Node(Node::from_path(path, leaf))
                    }
                };

                Ok(Spanned::new(repr, span))
            }
            _ => bail!(ty=> Error::UnsupportedType),
        }
    }

    pub(crate) fn as_leaf(&self) -> Leaf {
        match self {
            Tree::Node(node) => **node,
            Tree::Leaf(leaf) => *leaf,
        }
    }

    pub(crate) fn as_native(&self) -> Native {
        self.as_leaf().as_native()
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

impl ToTokens for Tree<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Tree::Node(node) => node.to_tokens(tokens),
            Tree::Leaf(leaf) => leaf.to_tokens(tokens),
        }
    }
}

impl From<Leaf> for Tree<'_> {
    fn from(leaf: Leaf) -> Self {
        Self::Leaf(leaf)
    }
}
