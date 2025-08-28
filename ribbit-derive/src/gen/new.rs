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
use crate::lift::Lift as _;
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
            let ty_struct_loose = ir.tight().loosen();
            let ty_struct = ty::Tree::from(ir.tight().clone());

            Or::R(variants.iter().map(move |variant| {
                assert!(!variant.extract, "TODO");

                let new = format_ident!(
                    "{}_{}",
                    new,
                    variant.r#struct.unpacked.to_string().to_snake_case(),
                );

                new_struct(&vis, &new, &variant.r#struct, |value| {
                    let ty_variant = variant.r#struct.tight.clone();

                    (((variant.discriminant.lift() % ty_struct_loose)
                        | Box::new(
                            (value.lift() % ty_variant % ty_struct_loose) << discriminant_size,
                        ))
                        % ty_struct.clone())
                    .to_token_stream()
                })
            }))
        }
    }
}

fn new_struct<F: FnOnce(TokenStream) -> TokenStream>(
    vis: &syn::Visibility,
    new: &syn::Ident,
    r#struct: &ir::Struct,
    map: F,
) -> TokenStream {
    let ty_struct = ty::Tree::from(r#struct.tight.clone());
    let ty_struct_loose = r#struct.tight.loosen();

    let parameters = r#struct.iter_nonzero().map(|field| {
        let ident = field.ident.escaped();
        let ty = field.ty.packed();
        quote!(#ident: #ty)
    });

    let value = match r#struct.is_newtype() {
        true => r#struct.iter_nonzero().fold(quote!(), |_, field| {
            let ident = field.ident.escaped().to_token_stream();
            match field.ty.is_leaf() {
                true => ident,
                false => (ident.lift() % (*field.ty).clone() % ty_struct.clone()).to_token_stream(),
            }
        }),
        false => {
            r#struct
                .iter_nonzero()
                .map(
                    |ir::Field {
                         ident, ty, offset, ..
                     }| {
                        (
                            ident.escaped().to_token_stream().lift(),
                            ty.deref().clone(),
                            *offset,
                        )
                    },
                )
                .fold(
                    Box::new(0usize.lift() % ty_struct_loose) as Box<dyn lift::Loosen>,
                    |state, (ident, ty_field, offset)| {
                        #[allow(clippy::precedence)]
                        Box::new((ident % ty_field % ty_struct_loose << offset) | state)
                    },
                )
                % ty_struct
        }
        .to_token_stream(),
    };

    let value = map(value);

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
