use bitvec::bitbox;
use darling::ast::Style;
use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;
use syn::TypePath;

use crate::error::bail;
use crate::input;
use crate::leaf::Leaf;
use crate::Spanned;

pub(crate) fn new<'input>(
    attr: &'input input::Attr,
    input: &'input syn::DeriveInput,
    item: &'input input::Item,
) -> darling::Result<Struct<'input>> {
    match &item.data {
        darling::ast::Data::Enum(_) => todo!(),
        darling::ast::Data::Struct(r#struct) => {
            match r#struct.style {
                Style::Unit | Style::Tuple => todo!(),
                Style::Struct => (),
            }

            let mut bits = bitbox![0; *attr.size];
            let mut fields = Vec::new();

            for field in &r#struct.fields {
                let uninit = FieldUninit::new(field);
                let size = uninit.size();
                let offset = match uninit.offset() {
                    Offset::Implicit => match bits.first_zero() {
                        Some(offset) => offset,
                        None => bail!(field => crate::Error::Overflow {
                            offset: 0,
                            available: 0,
                            required: size
                        }),
                    },
                };

                let prefix = &mut bits[offset..];
                match prefix.first_one().unwrap_or_else(|| prefix.len()) {
                    len if len < size => bail!(field => crate::Error::Overflow {
                        offset,
                        available: len,
                        required: size
                    }),
                    _ => prefix[..size].fill(true),
                }

                fields.push(uninit.with_offset(offset));
            }

            if bits.not_all() {
                bail!(input => crate::Error::Underflow {
                    bits,
                })
            }

            Ok(Struct {
                repr: attr
                    .size
                    .map_ref(|size| Leaf::new(attr.nonzero.unwrap_or(false), *size))
                    .into(),
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            })
        }
    }
}

pub(crate) struct Struct<'input> {
    repr: Spanned<Leaf>,
    attrs: &'input [syn::Attribute],
    vis: &'input syn::Visibility,
    ident: &'input syn::Ident,
    fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    pub(crate) fn repr(&self) -> &Spanned<Leaf> {
        &self.repr
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub(crate) fn fields(&self) -> &[Field] {
        &self.fields
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = &self.repr;
        let ident = self.ident;
        let vis = self.vis;
        let attrs = self.attrs;

        quote! {
            #( #attrs )*
            #vis struct #ident {
                value: #repr,
            }
        }
        .to_tokens(tokens)
    }
}

pub(crate) type Field<'input> = FieldInner<'input, usize>;
pub(crate) type FieldUninit<'input> = FieldInner<'input, Offset>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Offset {
    Implicit,
}

pub(crate) struct FieldInner<'input, O> {
    vis: &'input syn::Visibility,
    ident: Option<&'input syn::Ident>,
    repr: Spanned<Tree<'input>>,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(field: &'input input::Field) -> Self {
        Self {
            vis: &field.vis,
            ident: field.ident.as_ref(),
            repr: Tree::from_ty(&field.ty, field.nonzero, field.size),
            offset: Offset::Implicit,
        }
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            ident: self.ident,
            repr: self.repr,
            offset,
        }
    }
}

impl<'input, O: Copy> FieldInner<'input, O> {
    pub(crate) fn offset(&self) -> O {
        self.offset
    }
}

impl<'input, O: Copy> FieldInner<'input, O> {
    pub(crate) fn size(&self) -> usize {
        match &*self.repr {
            Tree::Node(node) => node.size(),
            Tree::Leaf(leaf) => leaf.size(),
        }
    }

    pub(crate) fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    pub(crate) fn repr(&self) -> &Spanned<Tree> {
        &self.repr
    }

    pub(crate) fn ident(&self) -> Option<&syn::Ident> {
        self.ident
    }

    pub(crate) fn nonzero(&self) -> bool {
        self.repr.nonzero()
    }
}

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
