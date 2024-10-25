use std::borrow::Cow;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use darling::util::SpannedValue;
use darling::FromMeta;
use quote::format_ident;

use crate::error::bail;
use crate::gen;
use crate::input;
use crate::ty;
use crate::ty::leaf;
use crate::ty::Leaf;
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

            let fields = r#struct
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| Field::new(&mut bits, index, field))
                .collect::<Result<Vec<_>, _>>()?;

            if bits.not_all() {
                bail!(input => crate::Error::Underflow {
                    bits,
                })
            }

            let leaf = Leaf::new(
                attr.nonzero
                    .map(Spanned::from)
                    .unwrap_or_else(|| false.into()),
                attr.size.into(),
            );

            if let (true, leaf::Repr::Arbitrary(_)) = (*leaf.nonzero, *leaf.repr) {
                bail!(leaf.nonzero=> crate::Error::ArbitraryNonZero);
            }

            if *leaf.nonzero && fields.iter().all(|field| !*field.ty.nonzero()) {
                bail!(leaf.nonzero=> crate::Error::StructNonZero);
            }

            Ok(Struct {
                repr: leaf.into(),
                opt: &attr.opt,
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            })
        }
    }
}

pub(crate) struct Struct<'input> {
    pub(crate) repr: Spanned<Leaf>,
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) vis: &'input syn::Visibility,
    pub(crate) ident: &'input syn::Ident,
    pub(crate) fields: Vec<Field<'input>>,
    pub(crate) opt: &'input StructOpt,
}

#[derive(FromMeta, Debug)]
pub(crate) struct StructOpt {
    #[darling(default)]
    pub(crate) new: gen::new::StructOpt,
}

pub(crate) struct Field<'input> {
    pub(crate) vis: &'input syn::Visibility,
    pub(crate) ident: FieldIdent<'input>,
    pub(crate) ty: Spanned<ty::Tree>,
    pub(crate) offset: usize,
    pub(crate) opt: &'input FieldOpt,
}

impl<'input> Field<'input> {
    fn new(
        bits: &mut BitBox,
        index: usize,
        field: &'input SpannedValue<input::Field>,
    ) -> darling::Result<Self> {
        let repr = ty::Tree::from_ty(
            field.ty.clone(),
            field.nonzero.map(Spanned::from),
            field.size.map(Spanned::from),
        )?;

        let size = *repr.size();

        let offset = match field.offset {
            None => match bits.first_zero() {
                Some(offset) => Spanned::new(offset, field.span()),
                None => bail!(field => crate::Error::Overflow {
                    offset: 0,
                    available: 0,
                    required: size
                }),
            },
            Some(offset) => match *offset >= bits.len() {
                false => offset.into(),
                true => bail!(field => crate::Error::Overflow {
                    offset: *offset,
                    available: 0,
                    required: size,
                }),
            },
        };

        let hole = &mut bits[*offset..];
        match hole.first_one().unwrap_or_else(|| hole.len()) {
            len if len < size => bail!(offset=> crate::Error::Overflow {
                offset: *offset,
                available: len,
                required: size
            }),
            _ => hole[..size].fill(true),
        }

        Ok(Self {
            vis: &field.vis,
            ident: FieldIdent::new(index, field.ident.as_ref()),
            ty: repr,
            offset: *offset,
            opt: &field.opt,
        })
    }
}

#[derive(FromMeta, Debug)]
pub(crate) struct FieldOpt {
    #[darling(default)]
    pub(crate) debug: gen::debug::FieldOpt,
}

pub(crate) enum FieldIdent<'input> {
    Named(&'input syn::Ident),
    Unnamed(usize),
}

impl<'input> FieldIdent<'input> {
    fn new(index: usize, ident: Option<&'input syn::Ident>) -> Self {
        ident
            .map(FieldIdent::Named)
            .unwrap_or_else(|| FieldIdent::Unnamed(index))
    }

    pub(crate) fn unescaped(&self, prefix: &'static str) -> syn::Ident {
        match self {
            FieldIdent::Named(named) => format_ident!("{}_{}", prefix, named),
            FieldIdent::Unnamed(unnamed) => format_ident!("{}_{}", prefix, unnamed),
        }
    }

    pub(crate) fn escaped(&self) -> Cow<syn::Ident> {
        match self {
            FieldIdent::Named(named) => Cow::Borrowed(*named),
            FieldIdent::Unnamed(unnamed) => Cow::Owned(format_ident!("_{}", unnamed)),
        }
    }
}
