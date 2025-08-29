use std::borrow::Cow;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use darling::usage::GenericsExt;
use darling::util::AsShape as _;
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
use crate::ty::Type;
use crate::Spanned;

pub(crate) fn new<'input>(item: &'input input::Item) -> darling::Result<Ir<'input>> {
    let ty_params = item.generics.declared_type_params();
    let mut bounds = Punctuated::new();

    let data = match &item.data {
        darling::ast::Data::Enum(variants) => {
            let Some(size) = item.opt.size.map(Spanned::from) else {
                bail!(Span::call_site()=> crate::Error::TopLevelSize);
            };

            let tight = match Tight::from_size(
                item.opt.nonzero.map(|value| *value).unwrap_or(false),
                *size,
            ) {
                Ok(tight) => tight,
                // FIXME: span
                Err(error) => bail!(item.opt.nonzero.unwrap()=> error),
            };

            // FIXME: support arbitrary discriminant
            let size_discriminant = variants.len().next_power_of_two().trailing_zeros() as usize;

            let variants = variants
                .iter()
                // FIXME: support arbitrary discriminant
                .enumerate()
                .map(|(discriminant, variant)| {
                    let r#struct = Struct::new(
                        &ty_params,
                        &mut bounds,
                        &variant.opt,
                        &variant.attrs,
                        &variant.ident,
                        &variant.fields,
                    )?;

                    if r#struct.r#type.as_tight().size() > (*size - size_discriminant) {
                        bail!(variant=> crate::Error::VariantSize {
                            variant: r#struct.r#type.as_tight().size(),
                            r#enum: *size,
                            discriminant: size_discriminant,
                        });
                    }

                    Ok(Variant {
                        extract: variant.extract,
                        discriminant,
                        r#struct,
                    })
                })
                .collect::<darling::Result<Vec<_>>>()?;

            let unpacked = &item.ident;
            let r#enum = Enum {
                opt: &item.opt,
                attrs: &item.attrs,
                packed: format_ident!("_{}Packed", item.ident),
                unpacked,
                r#type: Type::User {
                    path: parse_quote!(#unpacked),
                    uses: Default::default(),
                    tight,
                    exact: true,
                },
                variants,
            };

            Data::Enum(r#enum)
        }
        darling::ast::Data::Struct(r#struct) => Struct::new(
            &ty_params,
            &mut bounds,
            &item.opt,
            &item.attrs,
            &item.ident,
            r#struct,
        )
        .map(Data::Struct)?,
    };

    Ok(Ir {
        vis: &item.vis,
        generics: &item.generics,
        bounds,
        data,
    })
}

pub(crate) struct Ir<'input> {
    pub(crate) vis: &'input syn::Visibility,
    generics: &'input syn::Generics,
    bounds: Punctuated<syn::WherePredicate, syn::Token![,]>,
    pub(crate) data: Data<'input>,
}

impl Ir<'_> {
    pub(crate) fn generics(&self) -> &syn::Generics {
        self.generics
    }

    pub(crate) fn ident_packed(&self) -> &syn::Ident {
        match &self.data {
            Data::Enum(r#enum) => &r#enum.packed,
            Data::Struct(r#struct) => &r#struct.packed,
        }
    }

    pub(crate) fn ident_unpacked(&self) -> &syn::Ident {
        match &self.data {
            Data::Enum(r#enum) => &r#enum.unpacked,
            Data::Struct(r#struct) => &r#struct.unpacked,
        }
    }

    pub(crate) fn attrs(&self) -> &[syn::Attribute] {
        match &self.data {
            Data::Enum(r#enum) => r#enum.attrs,
            Data::Struct(r#struct) => r#struct.attrs,
        }
    }

    pub(crate) fn opt(&self) -> &StructOpt {
        match &self.data {
            Data::Enum(r#enum) => r#enum.opt,
            Data::Struct(r#struct) => r#struct.opt,
        }
    }

    pub(crate) fn r#type(&self) -> &Type {
        match &self.data {
            Data::Enum(r#enum) => &r#enum.r#type,
            Data::Struct(r#struct) => &r#struct.r#type,
        }
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
    pub(crate) opt: &'input StructOpt,
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) unpacked: &'input syn::Ident,
    pub(crate) packed: syn::Ident,
    pub(crate) r#type: Type,
    pub(crate) variants: Vec<Variant<'input>>,
}

impl Enum<'_> {
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
    pub(crate) extract: bool,
    pub(crate) discriminant: usize,
    pub(crate) r#struct: Struct<'input>,
}

pub(crate) struct Struct<'input> {
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) unpacked: &'input syn::Ident,
    pub(crate) packed: syn::Ident,
    pub(crate) r#type: Type,
    pub(crate) opt: &'input StructOpt,

    pub(crate) shape: darling::util::Shape,
    pub(crate) fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    fn new<'input>(
        ty_params: &darling::usage::IdentSet,
        bounds: &mut Punctuated<syn::WherePredicate, syn::Token![,]>,
        opt: &'input StructOpt,
        attrs: &'input [syn::Attribute],
        ident: &'input syn::Ident,
        fields: &'input darling::ast::Fields<SpannedValue<input::Field>>,
    ) -> darling::Result<Struct<'input>> {
        let Some(size) = opt.size.map(Spanned::from) else {
            bail!(Span::call_site()=> crate::Error::TopLevelSize);
        };

        let tight = match Tight::from_size(opt.nonzero.map(|value| *value).unwrap_or(false), *size)
        {
            Ok(tight) => tight,
            // FIXME: span
            Err(error) => bail!(opt.nonzero.unwrap()=> error),
        };

        let mut bits = bitbox![0; *size];
        let newtype = fields.len() == 1;

        let shape = fields.as_shape();
        let fields = fields
            .iter()
            .enumerate()
            .map(|(index, field)| Field::new(opt, ty_params, &mut bits, newtype, index, field))
            .collect::<Result<Vec<_>, _>>()?;

        if tight.is_nonzero() && fields.iter().all(|field| !field.ty.is_nonzero()) {
            bail!(opt.nonzero.unwrap()=> crate::Error::StructNonZero);
        }

        bounds.extend(
            fields
                .iter()
                .map(|field| &field.ty)
                .filter(|ty| ty.is_user())
                .filter(|ty| ty.size_expected() != 0)
                .map(|ty| -> syn::WherePredicate { parse_quote!(#ty: ::ribbit::Pack) }),
        );

        let unpacked = ident;
        Ok(Struct {
            attrs,
            packed: format_ident!("_{}Packed", ident),
            unpacked,
            r#type: Type::User {
                path: parse_quote!(#unpacked),
                uses: Default::default(),
                tight,
                exact: true,
            },
            opt,
            shape,
            fields,
        })
    }

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
    pub(crate) opt: &'input FieldOpt,
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) vis: &'input syn::Visibility,
    pub(crate) ident: FieldIdent<'input>,
    pub(crate) ty: Spanned<ty::Type>,
    pub(crate) offset: usize,
}

impl<'input> Field<'input> {
    fn new(
        opt: &StructOpt,
        ty_params: &darling::usage::IdentSet,
        bits: &mut BitBox,
        newtype: bool,
        index: usize,
        field: &'input SpannedValue<input::Field>,
    ) -> darling::Result<Self> {
        let ty = ty::Type::parse(newtype, opt, &field.opt, ty_params, field.ty.clone())?;

        let size = ty.size_expected();

        // Special-case ZSTs
        if size == 0 {
            return Ok(Self {
                opt: &field.opt,
                attrs: &field.attrs,
                vis: &field.vis,
                ident: FieldIdent::new(index, field.ident.as_ref()),
                ty,
                offset: bits.len(),
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
            opt: &field.opt,
            attrs: &field.attrs,
            vis: &field.vis,
            ident: FieldIdent::new(index, field.ident.as_ref()),
            ty,
            offset: *offset,
        })
    }
}

#[derive(FromMeta, Clone, Debug, Default)]
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

impl ToTokens for FieldIdent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.unescaped("").to_tokens(tokens)
    }
}
