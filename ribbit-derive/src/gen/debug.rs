use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;

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
            let fields = fields.iter().map(|field| {
                let name = field.ident.escaped();
                let opt = &field.opt.debug;

                let value = lift::lift(quote!(self.#name()), (*field.ty).clone()).to_token_stream();

                let value = match &opt.format {
                    None => value,
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
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = r#enum.unpacked(ident);

            let variants = variants.iter().map(|variant| {
                let name = variant.ident;
                match variant.ty.is_some() {
                    true => quote!(Self::#name(variant) => ::core::fmt::Debug::fmt(variant, f)),
                    false => quote!(write!(f, stringify!(#name))),
                }
            });

            quote! {
                impl ::core::fmt::Debug for #ident {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        ::core::fmt::Debug::fmt(&self.unpack(), f)
                    }
                }

                impl ::core::fmt::Debug for #unpacked {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        match self { #(#variants)* }
                    }
                }
            }
        }
    }
}
