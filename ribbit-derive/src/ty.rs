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
pub enum Tree {
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
                        if let Some(expected) = size.filter(|size| **size != *tight.size()) {
                            bail!(span=> Error::WrongSize {
                                ty: tight,
                                expected: *expected,
                                actual: *tight.size(),
                            });
                        }

                        Self::Leaf(tight)
                    }
                    None => {
                        let Some(size) = size else {
                            bail!(span=> Error::OpaqueSize);
                        };
                        let tight = Tight::from_size(nonzero.unwrap_or_else(|| false.into()), size);
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

    pub(crate) fn tighten(&self) -> Tight {
        match self {
            Tree::Node(node) => **node,
            Tree::Leaf(leaf) => *leaf,
        }
    }

    pub(crate) fn loosen(&self) -> Loose {
        self.tighten().loosen()
    }

    pub(crate) fn size_expected(&self) -> Spanned<usize> {
        match self {
            Tree::Node(node) => node.size(),
            Tree::Leaf(leaf) => leaf.size(),
        }
    }

    pub(crate) fn size_actual(&self) -> TokenStream {
        match *self.size_expected() {
            // Avoid requiring that ZSTs implement `ribbit::Pack`
            0 => quote!(::core::mem::size_of::<#self>()),
            _ => quote!(<#self as ::ribbit::Pack>::BITS),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Tree::Node(node) => node.mask(),
            Tree::Leaf(leaf) => leaf.mask(),
        }
    }

    pub(crate) fn nonzero(&self) -> bool {
        match self {
            Tree::Node(node) => *node.nonzero,
            Tree::Leaf(leaf) => *leaf.nonzero,
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
