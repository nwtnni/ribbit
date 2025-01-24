use std::borrow::Cow;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use darling::util::AsShape as _;
use darling::util::Shape;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote;
use syn::punctuated::Punctuated;

use crate::error::bail;
use crate::gen;
use crate::input;
use crate::ty;
use crate::ty::tight;
use crate::ty::Tight;
use crate::Spanned;

pub(crate) fn new<'a>(item: &'a input::Item, parent: Option<&'a Ir>) -> darling::Result<Ir<'a>> {
    let Some(size) = item.opt.size.map(Spanned::from) else {
        bail!(Span::call_site()=> crate::Error::TopLevelSize);
    };

    let tight = Tight::from_size(
        item.opt
            .nonzero
            .map(Spanned::from)
            .unwrap_or_else(|| false.into()),
        size,
    );

    if let (true, tight::Repr::Arbitrary(_)) = (*tight.nonzero, *tight.repr) {
        bail!(tight.nonzero=> crate::Error::ArbitraryNonZero);
    }

    let (_, generics_ty, _) = item.generics.split_for_impl();
    let mut bounds: Punctuated<syn::WherePredicate, syn::Token![,]> = parse_quote!();

    let data = match &item.data {
        darling::ast::Data::Enum(variants) => {
            let variants = variants
                .iter()
                .map(|variant| {
                    let ty = match variant.fields.as_shape() {
                        Shape::Unit => None,
                        Shape::Newtype => ty::Tree::parse(
                            variant.fields.fields[0].ty.clone(),
                            variant.opt.nonzero.map(Spanned::from),
                            variant.opt.size.map(Spanned::from),
                        )
                        .map(Some)?,
                        Shape::Named | Shape::Tuple => {
                            let ident = &variant.ident;
                            ty::Tree::parse(
                                parse_quote!(#ident #generics_ty),
                                variant.opt.nonzero.map(Spanned::from),
                                variant.opt.size.map(Spanned::from),
                            )
                            .map(Some)?
                        }
                    };

                    if let Some(ty) = ty.as_ref().filter(|ty| ty.is_node()) {
                        bounds.push(parse_quote!(#ty: ::ribbit::Pack));
                    }

                    Ok(Variant {
                        ident: &variant.ident,
                        ty: ty.map(Spanned::from),
                    })
                })
                .collect::<darling::Result<_>>()?;

            Data::Enum(Enum { variants })
        }
        darling::ast::Data::Struct(r#struct) => {
            let mut bits = bitbox![0; *size];

            let fields = match r#struct.fields.as_slice() {
                [field] => vec![Field::new(&mut bits, 0, Some(&item.opt), field)?],
                fields => fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| Field::new(&mut bits, index, None, field))
                    .collect::<Result<Vec<_>, _>>()?,
            };

            if *tight.nonzero && fields.iter().all(|field| !field.ty.nonzero()) {
                bail!(tight.nonzero=> crate::Error::StructNonZero);
            }

            fields
                .iter()
                .map(|field| &field.ty)
                .filter(|ty| ty.is_node())
                .filter(|ty| *ty.size_expected() != 0)
                .for_each(|ty| bounds.push(parse_quote!(#ty: ::ribbit::Pack)));

            Data::Struct(Struct { fields })
        }
    };

    Ok(Ir {
        tight: tight.into(),
        opt: &item.opt,
        attrs: &item.attrs,
        vis: &item.vis,
        ident: &item.ident,
        generics: &item.generics,
        bounds,
        data,
        parent,
    })
}

pub(crate) struct Ir<'input> {
    pub(crate) tight: Spanned<Tight>,
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) vis: &'input syn::Visibility,
    pub(crate) ident: &'input syn::Ident,
    generics: &'input syn::Generics,
    pub(crate) data: Data<'input>,
    bounds: Punctuated<syn::WherePredicate, syn::Token![,]>,
    pub(crate) opt: &'input StructOpt,
    pub(crate) parent: Option<&'input Ir<'input>>,
}

impl Ir<'_> {
    pub(crate) fn generics(&self) -> &syn::Generics {
        self.generics
    }

    pub(crate) fn generics_bounded(&self, extra: Option<syn::TypeParamBound>) -> syn::Generics {
        let mut generics = (*self.generics).clone();
        let r#where = generics.make_where_clause();

        for mut bound in self.bounds.clone() {
            if let (syn::WherePredicate::Type(ty), Some(extra)) = (&mut bound, &extra) {
                ty.bounds.push(extra.clone());
            };

            r#where.predicates.push(bound);
        }

        generics
    }
}

pub(crate) enum Data<'input> {
    Enum(Enum<'input>),
    Struct(Struct<'input>),
}

pub(crate) struct Enum<'input> {
    pub(crate) variants: Vec<Variant<'input>>,
}

impl Enum<'_> {
    pub(crate) fn unpacked(ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}Unpacked", ident)
    }

    pub(crate) fn discriminant_size(&self) -> usize {
        self.variants.len().next_power_of_two().trailing_zeros() as usize
    }

    pub(crate) fn discriminant_mask(&self) -> usize {
        crate::ty::Tight::from_size(false.into(), self.discriminant_size().into()).mask()
    }
}

pub(crate) struct Variant<'input> {
    pub(crate) ident: &'input syn::Ident,
    pub(crate) ty: Option<Spanned<ty::Tree>>,
}

pub(crate) struct Struct<'input> {
    pub(crate) fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    pub(crate) fn is_named(&self) -> bool {
        self.fields().any(|field| field.ident.is_named())
    }

    pub(crate) fn is_newtype(&self) -> bool {
        self.fields().count() == 1
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields
            .iter()
            .filter(|field| *field.ty.size_expected() != 0)
    }
}

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt {
    pub(crate) size: Option<SpannedValue<usize>>,
    pub(crate) nonzero: Option<SpannedValue<bool>>,

    #[darling(default)]
    pub(crate) new: gen::new::StructOpt,
    pub(crate) debug: Option<gen::debug::StructOpt>,
    pub(crate) eq: Option<gen::eq::StructOpt>,
    pub(crate) ord: Option<gen::ord::StructOpt>,
    pub(crate) hash: Option<gen::hash::StructOpt>,
    pub(crate) from: Option<gen::from::StructOpt>,
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
        forward: Option<&StructOpt>,
        field: &'input SpannedValue<input::Field>,
    ) -> darling::Result<Self> {
        let ty = ty::Tree::parse(
            field.ty.clone(),
            field
                .opt
                .nonzero
                .or_else(|| forward.and_then(|opt| opt.nonzero))
                .map(Spanned::from),
            field
                .opt
                .size
                .or_else(|| forward.and_then(|opt| opt.size))
                .map(Spanned::from),
        )?;

        let size = *ty.size_expected();

        // Special-case ZSTs
        if size == 0 {
            return Ok(Self {
                vis: &field.vis,
                ident: FieldIdent::new(index, field.ident.as_ref()),
                ty,
                offset: bits.len(),
                opt: &field.opt,
            });
        }

        let offset = match field.opt.offset {
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
            ty,
            offset: *offset,
            opt: &field.opt,
        })
    }
}

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct FieldOpt {
    pub(crate) nonzero: Option<SpannedValue<bool>>,
    pub(crate) size: Option<SpannedValue<usize>>,
    pub(crate) offset: Option<SpannedValue<usize>>,

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

    pub(crate) fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
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
