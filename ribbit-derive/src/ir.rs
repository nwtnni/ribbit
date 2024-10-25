use std::borrow::Cow;

use bitvec::bitbox;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::error::bail;
use crate::input;
use crate::repr::Leaf;
use crate::repr::Tree;
use crate::Spanned;

pub(crate) fn new<'input>(
    attr: &'input input::Attr,
    input: &'input syn::DeriveInput,
    item: &'input input::Item,
) -> darling::Result<Struct<'input>> {
    match &item.data {
        darling::ast::Data::Enum(_) => todo!(),
        darling::ast::Data::Struct(r#struct) => {
            let mut bits = bitbox![0; *attr.size];
            let mut fields = Vec::new();

            for (index, field) in r#struct.fields.iter().enumerate() {
                let uninit = FieldUninit::new(index, field);
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
    name: FieldName<'input>,
    repr: Spanned<Tree<'input>>,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(index: usize, field: &'input input::Field) -> Self {
        Self {
            vis: &field.vis,
            name: FieldName::new(index, field.ident.as_ref()),
            repr: Tree::from_ty(&field.ty, field.nonzero, field.size),
            offset: Offset::Implicit,
        }
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            name: self.name,
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

    pub(crate) fn name(&self) -> &FieldName {
        &self.name
    }

    pub(crate) fn nonzero(&self) -> bool {
        self.repr.nonzero()
    }
}

pub(crate) enum FieldName<'input> {
    Named(&'input syn::Ident),
    Unnamed(usize),
}

impl<'input> FieldName<'input> {
    fn new(index: usize, name: Option<&'input syn::Ident>) -> Self {
        name.map(FieldName::Named)
            .unwrap_or_else(|| FieldName::Unnamed(index))
    }

    pub(crate) fn unescaped(&self, prefix: &'static str) -> syn::Ident {
        match self {
            FieldName::Named(named) => format_ident!("{}_{}", prefix, named),
            FieldName::Unnamed(unnamed) => format_ident!("{}_{}", prefix, unnamed),
        }
    }

    pub(crate) fn escaped(&self) -> Cow<syn::Ident> {
        match self {
            FieldName::Named(named) => Cow::Borrowed(*named),
            FieldName::Unnamed(unnamed) => Cow::Owned(format_ident!("_{}", unnamed)),
        }
    }
}
