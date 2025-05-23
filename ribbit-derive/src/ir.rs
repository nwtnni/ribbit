use std::borrow::Cow;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use darling::usage::GenericsExt;
use darling::util::AsShape as _;
use darling::util::Shape;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::Literal;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::ToTokens;
use syn::parse_quote;
use syn::punctuated::Punctuated;

use crate::error::bail;
use crate::gen;
use crate::input;
use crate::ty;
use crate::ty::Tight;
use crate::Spanned;

pub(crate) fn new<'a>(item: &'a input::Item, parent: Option<&'a Ir>) -> darling::Result<Ir<'a>> {
    let Some(size) = item.opt.size.map(Spanned::from) else {
        bail!(Span::call_site()=> crate::Error::TopLevelSize);
    };

    let tight = match Tight::from_size(item.opt.nonzero.map(|value| *value).unwrap_or(false), *size)
    {
        Ok(tight) => tight,
        // FIXME: span
        Err(error) => bail!(item.opt.nonzero.unwrap()=> error),
    };

    let (_, generics_ty, _) = item.generics.split_for_impl();
    let ty_params = item.generics.declared_type_params();

    let mut bounds: Punctuated<syn::WherePredicate, syn::Token![,]> = parse_quote!();

    let data = match &item.data {
        darling::ast::Data::Enum(variants) => {
            let variants = variants
                .iter()
                .map(|variant| {
                    let ty = match variant.fields.as_shape() {
                        Shape::Unit => None,
                        Shape::Newtype => ty::Tree::parse(
                            &ty_params,
                            variant.fields.fields[0].ty.clone(),
                            variant.opt.nonzero.map(Spanned::from),
                            variant.opt.size.map(Spanned::from),
                        )
                        .map(Some)?,
                        Shape::Named | Shape::Tuple => {
                            let ident = &variant.ident;
                            ty::Tree::parse(
                                &ty_params,
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
            let newtype = r#struct.fields.len() == 1;

            let fields = r#struct
                .fields
                .iter()
                .map(|field| -> darling::Result<_> {
                    // For convenience, forward nonzero and size annotations
                    // for newtype structs.
                    let nonzero = match (newtype, field.opt.nonzero) {
                        (false, nonzero) | (true, nonzero @ Some(_)) => nonzero,
                        (true, None) => item.opt.nonzero,
                    };
                    let size = match (newtype, field.opt.size) {
                        (false, size) | (true, size @ Some(_)) => size,
                        (true, None) => item.opt.size,
                    };

                    let ty = ty::Tree::parse(
                        &ty_params,
                        field.ty.clone(),
                        nonzero.map(Spanned::from),
                        size.map(Spanned::from),
                    )?;

                    Ok((field, ty))
                })
                .enumerate()
                .map(|(index, field)| {
                    let (field, ty) = field?;
                    Field::new(ty, &mut bits, index, field)
                })
                .collect::<Result<Vec<_>, _>>()?;

            if tight.is_nonzero() && fields.iter().all(|field| !field.ty.is_nonzero()) {
                bail!(item.opt.nonzero.unwrap()=> crate::Error::StructNonZero);
            }

            fields
                .iter()
                .map(|field| &field.ty)
                .filter(|ty| ty.is_node())
                .filter(|ty| ty.size_expected() != 0)
                .for_each(|ty| bounds.push(parse_quote!(#ty: ::ribbit::Pack)));

            Data::Struct(Struct { fields })
        }
    };

    Ok(Ir {
        tight,
        item,
        bounds,
        data,
        parent,
    })
}

pub(crate) struct Ir<'input> {
    pub(crate) item: &'input input::Item,
    pub(crate) tight: Tight,
    pub(crate) data: Data<'input>,
    bounds: Punctuated<syn::WherePredicate, syn::Token![,]>,
    pub(crate) parent: Option<&'input Ir<'input>>,
}

impl Ir<'_> {
    pub(crate) fn generics(&self) -> &syn::Generics {
        &self.item.generics
    }

    pub(crate) fn generics_bounded(&self, extra: Option<syn::TypeParamBound>) -> syn::Generics {
        let mut generics = self.generics().clone();
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

    pub(crate) fn discriminant_mask(&self) -> u128 {
        crate::ty::Tight::from_size(false, self.discriminant_size())
            .unwrap()
            .mask()
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
        self.iter().any(|field| field.ident.is_named())
    }

    pub(crate) fn is_newtype(&self) -> bool {
        self.iter_nonzero().count() == 1
    }

    pub(crate) fn iter_nonzero(&self) -> impl Iterator<Item = &Field> {
        self.iter().filter(|field| field.ty.size_expected() != 0)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
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
        ty: Spanned<ty::Tree>,
        bits: &mut BitBox,
        index: usize,
        field: &'input SpannedValue<input::Field>,
    ) -> darling::Result<Self> {
        let size = ty.size_expected();

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
    pub(crate) fn new(index: usize, ident: Option<&'input syn::Ident>) -> Self {
        ident
            .map(FieldIdent::Named)
            .unwrap_or_else(|| FieldIdent::Unnamed(index))
    }

    pub(crate) fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }

    pub(crate) fn unescaped(&self, prefix: &'static str) -> TokenStream {
        match self {
            FieldIdent::Named(named) if prefix.is_empty() => (*named).clone(),
            FieldIdent::Unnamed(unnamed) if prefix.is_empty() => {
                return Literal::usize_unsuffixed(*unnamed).to_token_stream()
            }

            FieldIdent::Named(named) => format_ident!("{}_{}", prefix, named),
            FieldIdent::Unnamed(unnamed) => {
                format_ident!("{}_{}", prefix, unnamed)
            }
        }
        .to_token_stream()
    }

    pub(crate) fn escaped(&self) -> Cow<syn::Ident> {
        match self {
            FieldIdent::Named(named) => Cow::Borrowed(*named),
            FieldIdent::Unnamed(unnamed) => Cow::Owned(format_ident!("_{}", unnamed)),
        }
    }
}
