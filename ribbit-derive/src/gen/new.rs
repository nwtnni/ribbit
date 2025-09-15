use core::iter;

use darling::FromMeta;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::parse_quote;

use crate::ir;
use crate::lift;
use crate::Or;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt {
    vis: Option<syn::Visibility>,
    rename: Option<syn::Ident>,
}

pub(crate) fn new<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let vis = ir.opt().new.vis();

    let tight = ir.r#type().as_tight();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L(
            iter::once(new_struct(
                &vis,
                &ir.opt().new.name(None),
                r#struct,
                |expr| expr.compile(tight),
            ))
            .chain(iter::once(new_struct_unchecked(
                &vis,
                &ir.opt().new.name_unchecked(None),
                r#struct,
                |expr| expr.compile(tight),
            ))),
        ),
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => {
            let discriminant = r#enum.discriminant();

            Or::R(variants.iter().flat_map(move |variant| {
                let name = variant.r#struct.unpacked.to_string().to_snake_case();

                let compile = |expr: lift::Expr| {
                    lift::Expr::or([
                        lift::Expr::constant(variant.discriminant as u128),
                        expr.shift_left(discriminant.size as u8),
                    ])
                    .compile(tight)
                };

                [
                    new_struct(
                        &vis,
                        &ir.opt().new.name(Some(&name)),
                        &variant.r#struct,
                        compile,
                    ),
                    new_struct_unchecked(
                        &vis,
                        &ir.opt().new.name_unchecked(Some(&name)),
                        &variant.r#struct,
                        compile,
                    ),
                ]
            }))
        }
    }
}

fn new_struct<'ir, F: FnOnce(lift::Expr<'ir>) -> TokenStream>(
    vis: &syn::Visibility,
    new: &syn::Ident,
    r#struct: &'ir ir::Struct,
    compile: F,
) -> TokenStream {
    let parameters = r#struct.iter_nonzero().map(|field| {
        let ident = field.ident.escaped();
        let r#type = field.r#type.packed();
        quote!(#ident: #r#type)
    });

    let value = compile(lift::Expr::or(r#struct.iter_nonzero().map(|field| {
        lift::Expr::value(field.ident.escaped(), &field.r#type).shift_left(field.offset as u8)
    })));

    let precondition = crate::gen::pre::precondition();

    quote! {
        #[inline]
        #vis const fn #new(
            #(#parameters),*
        ) -> Self {
            #precondition
            Self {
                value: #value,
                r#type: ::ribbit::private::PhantomData,
            }
        }
    }
}

fn new_struct_unchecked<'ir, F: FnOnce(lift::Expr<'ir>) -> TokenStream>(
    vis: &syn::Visibility,
    new_unchecked: &syn::Ident,
    r#struct: &'ir ir::Struct,
    compile: F,
) -> TokenStream {
    let precondition = crate::gen::pre::precondition();
    let r#type = r#struct.r#type.as_tight();
    let value = compile(lift::Expr::value_tight(quote!(value), r#type));

    quote! {
        #[inline]
        #vis const unsafe fn #new_unchecked(value: #r#type) -> Self {
            #precondition
            Self {
                value: #value,
                r#type: ::ribbit::private::PhantomData,
            }
        }
    }
}

impl StructOpt {
    fn vis(&self) -> syn::Visibility {
        self.vis
            .clone()
            .unwrap_or(syn::Visibility::Public(parse_quote!(pub)))
    }

    pub(crate) fn name(&self, variant: Option<&str>) -> syn::Ident {
        let new = self.rename.clone().unwrap_or_else(|| format_ident!("new"));
        match variant {
            Some(variant) => format_ident!("{}_{}", new, variant),
            None => new,
        }
    }

    pub(crate) fn name_unchecked(&self, variant: Option<&str>) -> syn::Ident {
        let new = self.name(None);
        match variant {
            Some(variant) => format_ident!("{}_{}_unchecked", new, variant),
            None => format_ident!("{}_unchecked", new),
        }
    }
}
