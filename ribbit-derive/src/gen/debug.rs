use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens as _;

use crate::ir;
use crate::lift::Lift as _;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt {
    format: Option<syn::LitStr>,
}

pub(crate) fn debug(
    ir::Ir {
        opt,
        ident,
        generics,
        data,
        ..
    }: &ir::Ir,
) -> TokenStream {
    if opt.debug.is_none() {
        return TokenStream::new();
    }

    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            let fields = fields
                .iter()
                .filter(|field| *field.ty.size() != 0)
                .map(|field| {
                    let name = field.ident.escaped();
                    let opt = &field.opt.debug;

                    let value = quote!(self.#name()).lift() % (*field.ty).clone();

                    let value = match &opt.format {
                        None => value.to_token_stream(),
                        Some(format) => quote!(format_args!(#format, #value)),
                    };

                    quote! {
                        .field(stringify!(#name), &#value)
                    }
                });

            let (r#impl, ty, r#where) = generics.split_for_impl();
            quote! {
                impl #r#impl ::core::fmt::Debug for #ident #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        f.debug_struct(stringify!(#ident))
                            #(#fields)*
                            .finish()
                    }
                }
            }
        }
        ir::Data::Enum(r#enum @ ir::Enum { generics, variants }) => {
            let unpacked = r#enum.unpacked(ident);

            let variants = variants.iter().map(|variant| {
                let name = variant.ident;
                match variant.ty.is_some() {
                    true => quote!(Self::#name(variant) => ::core::fmt::Debug::fmt(variant, f)),
                    false => quote!(write!(f, stringify!(#name))),
                }
            });

            let (r#impl, ty, r#where) = generics.split_for_impl();
            quote! {
                impl #r#impl ::core::fmt::Debug for #ident #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        ::core::fmt::Debug::fmt(&self.unpack(), f)
                    }
                }

                impl #r#impl ::core::fmt::Debug for #unpacked #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        match self { #(#variants)* }
                    }
                }
            }
        }
    }
}
