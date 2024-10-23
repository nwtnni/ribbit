use bitvec::bitbox;
use darling::ast::Style;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

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
    repr: Spanned<Leaf>,
    nonzero: Option<bool>,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(field: &'input input::Field) -> Self {
        Self {
            vis: &field.vis,
            ident: field.ident.as_ref(),
            repr: Leaf::from_ty(&field.ty).unwrap(),
            nonzero: field.nonzero,
            offset: Offset::Implicit,
        }
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            ident: self.ident,
            repr: self.repr,
            nonzero: self.nonzero,
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
        self.repr.size()
    }

    pub(crate) fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    pub(crate) fn repr(&self) -> &Spanned<Leaf> {
        &self.repr
    }

    pub(crate) fn ident(&self) -> Option<&syn::Ident> {
        self.ident
    }
}
