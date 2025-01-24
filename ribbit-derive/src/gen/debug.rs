use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens as _;
use syn::parse_quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt {
    format: Option<syn::LitStr>,
}

pub(crate) fn debug(
    ir @ ir::Ir {
        opt, ident, data, ..
    }: &ir::Ir,
) -> TokenStream {
    if opt.debug.is_none() {
        return TokenStream::new();
    }

    // Debug implementation requires access to getters, which
    // requires stronger bounds
    let generics = ir.generics_bounded(Some(parse_quote!(::core::fmt::Debug)));
    let (r#impl, ty, r#where) = generics.split_for_impl();

    match data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.fields().map(|field| {
                let name = field.ident.escaped();
                let opt = &field.opt.debug;

                let value = quote!(self.#name());
                let value = match &opt.format {
                    None => value.to_token_stream(),
                    Some(format) => quote!(format_args!(#format, #value)),
                };

                match field.ident.is_named() {
                    true => quote!(stringify!(#name), &#value),
                    false => quote!(&#value),
                }
            });

            let debug = match r#struct.is_named() {
                true => quote!(debug_struct),
                false => quote!(debug_tuple),
            };

            quote! {
                impl #r#impl ::core::fmt::Debug for #ident #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        f.#debug(stringify!(#ident))
                            #(
                                .field(#fields)
                            )*
                            .finish()
                    }
                }
            }
        }
        ir::Data::Enum(ir::Enum { variants }) => {
            let unpacked = ir::Enum::unpacked(ident);

            let variants = variants.iter().map(|variant| {
                let name = variant.ident;
                match variant.ty.is_some() {
                    true => quote!(Self::#name(variant) => ::core::fmt::Debug::fmt(variant, f)),
                    false => quote!(Self::#name => write!(f, stringify!(#name))),
                }
            });

            quote! {
                impl #r#impl ::core::fmt::Debug for #ident #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        ::core::fmt::Debug::fmt(&self.unpack(), f)
                    }
                }

                impl #r#impl ::core::fmt::Debug for #unpacked #ty #r#where {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        match self { #(#variants),* }
                    }
                }
            }
        }
    }
}
