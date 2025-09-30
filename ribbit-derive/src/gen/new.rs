use core::iter;
use std::borrow::Cow;

use darling::FromMeta;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt(ir::CommonOpt);

pub(crate) fn new<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let vis = ir.opt().new.0.vis(ir.vis);

    let tight = ir.r#type().as_tight();

    match &ir.data {
        ir::Data::Struct(_) if ir.opt().new.0.skip => Or::L(iter::empty()),
        ir::Data::Struct(r#struct) => Or::R(Or::L(
            iter::once(new_struct(
                vis,
                &ir.opt().new.name(None),
                r#struct,
                |expr| expr.compile(tight),
            ))
            .chain(iter::once(new_struct_unchecked(
                vis,
                &ir.opt().new.name_unchecked(None),
                r#struct,
                |expr| expr.compile(tight),
            ))),
        )),
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => Or::R(Or::R(
            variants
                .iter()
                .filter(|variant| !variant.r#struct.opt.new.0.skip)
                .flat_map(move |variant| {
                    let suffix = variant.r#struct.unpacked.to_string().to_snake_case();

                    let compile = |expr: lift::Expr| {
                        lift::Expr::or([
                            lift::Expr::constant(variant.discriminant as u128),
                            expr.shift_left(r#enum.discriminant.size as u8),
                        ])
                        .compile(tight)
                    };

                    [
                        new_struct(
                            vis,
                            &ir.opt().new.name(Some(&suffix)),
                            &variant.r#struct,
                            compile,
                        ),
                        new_struct_unchecked(
                            vis,
                            &ir.opt().new.name_unchecked(Some(&suffix)),
                            &variant.r#struct,
                            compile,
                        ),
                    ]
                }),
        )),
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
    pub(crate) fn name<'ir>(&'ir self, suffix: Option<&'ir str>) -> Cow<'ir, syn::Ident> {
        self.0.rename_with(|| {
            Cow::Owned(match suffix {
                Some(variant) => format_ident!("new_{}", variant),
                None => format_ident!("new"),
            })
        })
    }

    fn name_unchecked<'ir>(&'ir self, suffix: Option<&'ir str>) -> Cow<'ir, syn::Ident> {
        let new = self.name(None);
        Cow::Owned(match suffix {
            Some(variant) => format_ident!("{}_{}_unchecked", new, variant),
            None => format_ident!("{}_unchecked", new),
        })
    }
}
