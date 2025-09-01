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
    let new = ir.opt().new.name();
    let tight = ir.r#type().as_tight();

    match &ir.data {
        ir::Data::Struct(r#struct) => {
            Or::L(core::iter::once(new_struct(&vis, &new, r#struct, |expr| {
                expr.compile(tight)
            })))
        }
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => {
            let discriminant = r#enum.discriminant();

            Or::R(variants.iter().map(move |variant| {
                assert!(!variant.extract, "TODO");

                let new = format_ident!(
                    "{}_{}",
                    new,
                    variant.r#struct.unpacked.to_string().to_snake_case(),
                );

                new_struct(&vis, &new, &variant.r#struct, |expr| {
                    lift::Expr::or([
                        lift::Expr::constant(variant.discriminant as u128),
                        expr.shift_left(discriminant.size as u8),
                    ])
                    .compile(tight)
                })
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
        let ty = field.ty.packed();
        quote!(#ident: #ty)
    });

    let value = compile(lift::Expr::or(r#struct.iter_nonzero().map(|field| {
        lift::Expr::value(field.ident.escaped(), &field.ty).shift_left(field.offset as u8)
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

impl StructOpt {
    fn vis(&self) -> syn::Visibility {
        self.vis
            .clone()
            .unwrap_or(syn::Visibility::Public(parse_quote!(pub)))
    }

    pub(crate) fn name(&self) -> syn::Ident {
        self.rename.clone().unwrap_or_else(|| format_ident!("new"))
    }
}
