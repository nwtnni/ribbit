use core::ops::Deref as _;

use darling::FromMeta;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::parse_quote;

use crate::ir;
use crate::lift;
use crate::ty;
use crate::Or;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt {
    vis: Option<syn::Visibility>,
    rename: Option<syn::Ident>,
}

pub(crate) fn new<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let vis = ir.opt().new.vis();
    let new = ir.opt().new.name();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L(core::iter::once(new_struct(
            &vis,
            &new,
            r#struct,
            core::convert::identity,
        ))),
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => {
            let discriminant_size = r#enum.discriminant_size();
            let ty_struct = ir.tight();

            Or::R(variants.iter().map(move |variant| {
                assert!(!variant.extract, "TODO");

                let new = format_ident!(
                    "{}_{}",
                    new,
                    variant.r#struct.unpacked.to_string().to_snake_case(),
                );

                new_struct(&vis, &new, &variant.r#struct, |value| {
                    lift::Expr::or(
                        ty_struct,
                        [
                            (0, lift::Expr::constant(variant.discriminant as u128)),
                            (discriminant_size as u8, value),
                        ],
                    )
                })
            }))
        }
    }
}

fn new_struct<'ir, F: FnOnce(lift::Expr<'ir>) -> lift::Expr<'ir>>(
    vis: &syn::Visibility,
    new: &syn::Ident,
    r#struct: &'ir ir::Struct,
    map: F,
) -> TokenStream {
    let ty_struct = &r#struct.tight;

    let parameters = r#struct.iter_nonzero().map(|field| {
        let ident = field.ident.escaped();
        let ty = field.ty.packed();
        quote!(#ident: #ty)
    });

    let value = lift::Expr::or(
        ty_struct,
        r#struct.iter_nonzero().map(
            |ir::Field {
                 ident, ty, offset, ..
             }| { (*offset as u8, lift::Expr::new(ident.escaped(), ty.deref())) },
        ),
    );

    let value = map(value).canonicalize();

    quote! {
        #[inline]
        #vis const fn #new(
            #(#parameters),*
        ) -> Self {
            let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
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
