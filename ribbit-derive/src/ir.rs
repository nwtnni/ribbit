use std::borrow::Cow;

use bitvec::bitbox;
use bitvec::boxed::BitBox;
use darling::usage::GenericsExt;
use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::Literal;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::parse_quote;
use syn::punctuated::Punctuated;

use crate::error::bail;
use crate::gen;
use crate::input;
use crate::r#type::Tight;
use crate::Spanned;
use crate::Type;

pub(crate) struct Ir<'input> {
    pub(crate) vis: &'input syn::Visibility,
    generics: &'input syn::Generics,
    generics_bounded: syn::Generics,
    pub(crate) data: Data<'input>,
}

impl<'input> Ir<'input> {
    pub(crate) fn new(item: &'input input::Item) -> darling::Result<Self> {
        let type_params = item.generics.declared_type_params();

        let mut generics_bounded = item.generics.clone();
        let generics_where = generics_bounded.make_where_clause();

        let data = match &item.data {
            darling::ast::Data::Enum(variants) => {
                let Some(size) = item.opt.size.map(Spanned::from) else {
                    bail!(Span::call_site()=> crate::Error::TopLevelSize);
                };

                let tight = match Tight::from_size(item.opt.nonzero.is_some(), *size) {
                    Ok(tight) => tight,
                    // FIXME: span
                    Err(error) => bail!(item.opt.nonzero.unwrap()=> error),
                };

                let mut current_discriminant = 0;

                let variants_ir = variants
                    .iter()
                    .map(|variant| {
                        let r#struct = Struct::new(
                            &type_params,
                            &mut generics_where.predicates,
                            &variant.opt,
                            &variant.ident,
                            &variant.fields,
                        )?;

                        // FIXME: support arbitrary expression
                        let discriminant = if let Some(syn::Expr::Lit(syn::ExprLit {
                            attrs: _,
                            lit: syn::Lit::Int(int),
                        })) = &variant.discriminant
                        {
                            int.base10_parse()?
                        } else if variant.discriminant.is_some() {
                            bail!(variant=> crate::Error::VariantDiscriminant);
                        } else {
                            current_discriminant
                        };

                        current_discriminant = discriminant + 1;

                        Ok(Variant {
                            discriminant,
                            r#struct,
                        })
                    })
                    .collect::<darling::Result<Vec<_>>>()?;

                if item.opt.nonzero.is_some() {
                    if let Some((variant, span)) = variants_ir
                        .iter()
                        .zip(variants)
                        .find(|(variant, _)| variant.discriminant == 0)
                    {
                        if variant.r#struct.opt.nonzero.is_none() {
                            bail!(span=> crate::Error::VariantNonZero);
                        }
                    }
                }

                let size_discriminant = variants_ir
                    .iter()
                    .map(|variant| variant.discriminant)
                    .max()
                    // Size must fit values 0..=discriminant
                    .map(|discriminant| discriminant + 1)
                    .unwrap_or(0)
                    .next_power_of_two()
                    .trailing_zeros() as usize;

                for (variant, span) in variants_ir.iter().zip(variants) {
                    let size_variant = variant.r#struct.r#type.as_tight().size();
                    if size_variant + size_discriminant > *size {
                        bail!(span=> crate::Error::VariantSize {
                            variant: r#variant.r#struct.r#type.as_tight().size(),
                            r#enum: *size,
                            discriminant: size_discriminant,
                        });
                    }
                }

                let unpacked = &item.ident;
                let r#enum = Enum {
                    opt: &item.opt,
                    packed: item.opt.packed.name(unpacked),
                    unpacked,
                    r#type: Type::User {
                        path: parse_quote!(#unpacked),
                        uses: Default::default(),
                        tight,
                    },
                    discriminant: Discriminant {
                        size: size_discriminant,
                        mask: crate::mask(size_discriminant),
                    },
                    variants: variants_ir,
                };

                Data::Enum(r#enum)
            }
            darling::ast::Data::Struct(r#struct) => Struct::new(
                &type_params,
                &mut generics_where.predicates,
                &item.opt,
                &item.ident,
                r#struct,
            )
            .map(Data::Struct)?,
        };

        Ok(Ir {
            vis: &item.vis,
            generics: &item.generics,
            generics_bounded,
            data,
        })
    }

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
            Data::Enum(r#enum) => r#enum.unpacked,
            Data::Struct(r#struct) => r#struct.unpacked,
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

    pub(crate) fn generics_bounded(&self) -> &syn::Generics {
        &self.generics_bounded
    }
}

pub(crate) enum Data<'input> {
    Enum(Enum<'input>),
    Struct(Struct<'input>),
}

pub(crate) struct Enum<'input> {
    pub(crate) opt: &'input StructOpt,
    pub(crate) unpacked: &'input syn::Ident,
    pub(crate) packed: syn::Ident,
    pub(crate) r#type: Type,
    pub(crate) discriminant: Discriminant,
    pub(crate) variants: Vec<Variant<'input>>,
}

#[derive(Debug)]
pub(crate) struct Discriminant {
    pub(crate) size: usize,
    pub(crate) mask: u128,
}

pub(crate) struct Variant<'input> {
    pub(crate) discriminant: usize,
    pub(crate) r#struct: Struct<'input>,
}

pub(crate) struct Struct<'input> {
    pub(crate) unpacked: &'input syn::Ident,
    pub(crate) packed: syn::Ident,
    pub(crate) r#type: Type,
    pub(crate) opt: &'input StructOpt,

    pub(crate) max_offset: usize,
    pub(crate) fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    fn new<'input>(
        type_params: &darling::usage::IdentSet,
        bounds: &mut Punctuated<syn::WherePredicate, syn::Token![,]>,
        opt: &'input StructOpt,
        unpacked: &'input syn::Ident,
        fields: &'input darling::ast::Fields<SpannedValue<input::Field>>,
    ) -> darling::Result<Struct<'input>> {
        let Some(size) = opt.size.map(Spanned::from) else {
            bail!(Span::call_site()=> crate::Error::TopLevelSize);
        };

        let tight = match Tight::from_size(opt.nonzero.is_some(), *size) {
            Ok(tight) => tight,
            // FIXME: span
            Err(error) => bail!(opt.nonzero.unwrap()=> error),
        };

        let mut bits = bitbox![0; *size];
        let newtype = fields.len() == 1;

        let fields = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                Field::new(opt, type_params, bounds, &mut bits, newtype, index, field)
            })
            .collect::<Result<Vec<_>, _>>()?;

        if tight.is_nonzero() && fields.iter().all(|field| !field.r#type.is_nonzero()) {
            bail!(opt.nonzero.unwrap()=> crate::Error::StructNonZero);
        }

        Ok(Struct {
            packed: opt.packed.name(unpacked),
            unpacked,
            r#type: Type::User {
                path: parse_quote!(#unpacked),
                uses: Default::default(),
                tight,
            },
            opt,
            max_offset: fields.iter().map(|field| field.offset).max().unwrap_or(0),
            fields,
        })
    }

    pub(crate) fn iter_nonzero(&self) -> impl Iterator<Item = &Field> {
        self.iter().filter(|field| field.r#type.size() != 0)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }
}

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt {
    pub(crate) size: Option<SpannedValue<usize>>,
    pub(crate) nonzero: Option<SpannedValue<()>>,

    #[darling(default)]
    pub(crate) packed: gen::packed::StructOpt,

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
    pub(crate) r#type: Spanned<Type>,
    pub(crate) offset: usize,
}

impl<'input> Field<'input> {
    fn new(
        opt: &StructOpt,
        type_params: &darling::usage::IdentSet,
        bounds: &mut Punctuated<syn::WherePredicate, syn::Token![,]>,
        bits: &mut BitBox,
        newtype: bool,
        index: usize,
        field: &'input SpannedValue<input::Field>,
    ) -> darling::Result<Self> {
        let r#type = Type::parse(newtype, opt, &field.opt, type_params, field.ty.clone())?;
        let size = r#type.size();

        // Gather trait bounds for generic type parameters
        if r#type.is_generic() {
            let r#type = &field.ty;
            bounds.push(parse_quote!(#r#type: ::ribbit::Pack));
        }

        // Avoid erroring on `bits.first_zero()` for ZST fields
        if size == 0 {
            return Ok(Self {
                vis: &field.vis,
                ident: FieldIdent::new(index, field.ident.as_ref()),
                r#type,
                offset: 0,
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
            r#type,
            offset: *offset,
        })
    }
}

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt {
    pub(crate) nonzero: Option<SpannedValue<()>>,
    pub(crate) size: Option<SpannedValue<usize>>,
    pub(crate) offset: Option<SpannedValue<usize>>,
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

    pub(crate) fn pattern(&self) -> TokenStream {
        match self {
            FieldIdent::Named(_) => quote!(#self),
            FieldIdent::Unnamed(_) => {
                let escaped = self.escaped();
                quote!(#self: #escaped)
            }
        }
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
