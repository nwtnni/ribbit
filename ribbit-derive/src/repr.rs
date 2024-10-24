mod arbitrary;
mod leaf;
mod native;

pub(crate) use arbitrary::Arbitrary;
use darling::util::SpannedValue;
pub(crate) use leaf::Leaf;
pub(crate) use native::Native;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;
use syn::TypePath;

use crate::Spanned;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Tree<'input> {
    Node(Node<'input>),
    Leaf(Leaf),
}

impl<'input> Tree<'input> {
    pub(crate) fn from_ty(
        ty: &'input syn::Type,
        nonzero: Option<bool>,
        size: Option<usize>,
    ) -> Spanned<Self> {
        match ty {
            syn::Type::Array(_) => todo!(),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Path(path) => {
                let span = path.span();
                let node = Leaf::from_path(path).map(Self::Leaf).unwrap_or_else(|| {
                    let repr = Leaf::new(
                        nonzero.unwrap_or(false),
                        size.expect("Opaque type requires size"),
                    );
                    Self::Node(Node::from_path(path, repr))
                });

                SpannedValue::new(node, span).into()
            }
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(_) => todo!(),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }

    pub(crate) fn as_native(&self) -> Leaf {
        match self {
            Tree::Node(node) => node.repr.as_native(),
            Tree::Leaf(leaf) => leaf.as_native(),
        }
    }

    pub(crate) fn convert_to_native<T: ToTokens>(&self, input: T) -> TokenStream {
        match self {
            Tree::Node(node) => node.repr.convert_to_native(quote!(::ribbit::pack(#input))),
            Tree::Leaf(leaf) => leaf.convert_to_native(input),
        }
    }

    pub(crate) fn convert_from_native<T: ToTokens>(&self, input: T) -> TokenStream {
        match self {
            Tree::Node(node) => {
                let value = node.repr.convert_from_native(input);
                quote!(::ribbit::unpack(#value))
            }
            Tree::Leaf(leaf) => leaf.convert_from_native(input),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Tree::Node(node) => node.repr.mask(),
            Tree::Leaf(leaf) => leaf.mask(),
        }
    }

    pub(crate) fn nonzero(&self) -> bool {
        match self {
            Tree::Node(node) => node.nonzero(),
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Node<'input> {
    path: &'input TypePath,
    repr: Leaf,
}

impl<'input> Node<'input> {
    fn from_path(path: &'input TypePath, repr: Leaf) -> Self {
        Self { path, repr }
    }

    pub(crate) fn size(&self) -> usize {
        self.repr.size()
    }

    pub(crate) fn nonzero(&self) -> bool {
        self.repr.nonzero
    }
}

impl ToTokens for Node<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}
