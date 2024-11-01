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

    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            let parameters = fields.iter().map(|field| {
                let ident = field.ident.escaped();
                let ty = &field.ty;
                quote!(#ident: #ty)
            });

            let value = fields.iter().fold(
                Box::new(0usize.lift() % tight.loosen()) as Box<dyn lift::Loosen>,
                |state, field| {
                    let ident = field.ident.escaped().to_token_stream();
                    let value =
                        (ident.lift() % (*field.ty).clone() % tight.loosen()) << field.offset;
                    Box::new(value | state)
                },
            ) % ty::Tree::from(**tight);

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

            let discriminant_size = r#enum.discriminant_size();
            let loose = tight.loosen();

            let discriminants = variants.iter().enumerate().map(|(index, variant)| {
                let packed = (index.lift() % loose)
                    | match &variant.ty {
                        None => Box::new(0.lift() % loose) as Box<dyn lift::Loosen>,
                        Some(ty) => Box::new(
                            ((quote!(inner).lift() % (**ty).clone()) << discriminant_size) % loose,
                        ),
                    };

                let ident = &variant.ident;
                match &variant.ty {
                    None => quote!(#unpacked::#ident => #packed),
                    Some(_) => quote!(#unpacked::#ident(inner) => #packed),
                }
            });

            let value = quote!(match unpacked { #(#discriminants),* }).lift()
                % loose
                % ty::Tree::from(**tight);

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
