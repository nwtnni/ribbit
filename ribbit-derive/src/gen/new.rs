use core::ops::Deref as _;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::Lift as _;
use crate::ty;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt {
    rename: Option<syn::Ident>,
    vis: Option<syn::Visibility>,
}

pub(crate) fn new(
    ir::Ir {
        ident,
        opt,
        tight,
        vis,
        data,
        ..
    }: &ir::Ir,
) -> TokenStream {
    let new = opt
        .new
        .rename
        .clone()
        .unwrap_or_else(|| format_ident!("new"));
    let vis = opt.new.vis.as_ref().unwrap_or(vis);

    let ty_struct = ty::Tree::from(**tight);
    let ty_struct_loose = tight.loosen();

    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            let parameters = fields.iter().map(|field| {
                let ident = field.ident.escaped();
                let ty = &field.ty;
                quote!(#ident: #ty)
            });

            let value = fields
                .iter()
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
                % ty_struct;

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
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = r#enum.unpacked(ident);
            let variants = variants
                .iter()
                .map(|ir::Variant { ident, ty, .. }| (ident, ty.as_deref()))
                .enumerate()
                .map(|(discriminant, (ident, ty))| {
                    #[allow(clippy::precedence)]
                    let packed = (discriminant.lift() % ty_struct_loose)
                        | match ty.cloned() {
                            None => Box::new(0.lift() % ty_struct_loose) as Box<dyn lift::Loosen>,
                            Some(ty_variant) => Box::new(
                                (quote!(inner).lift() % ty_variant << r#enum.discriminant_size())
                                    % ty_struct_loose,
                            ),
                        };

                    match ty {
                        None => quote!(#unpacked::#ident => #packed),
                        Some(_) => quote!(#unpacked::#ident(inner) => #packed),
                    }
                });

            let value =
                quote!(match unpacked { #(#variants),* }).lift() % ty_struct_loose % ty_struct;

            quote! {
                #[inline]
                #vis const fn #new(
                    unpacked: #unpacked,
                ) -> Self {
                    let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                    Self {
                        value: #value,
                        r#type: ::ribbit::private::PhantomData,
                    }
                }
            }
        }
    }
}
