use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;

#[derive(FromMeta, Debug)]
pub(crate) struct StructOpt;

#[derive(FromMeta, Debug, Default)]
pub(crate) struct FieldOpt {
    format: Option<syn::LitStr>,
}

pub(crate) fn debug(
    ir::Struct {
        fields, opt, ident, ..
    }: &ir::Struct,
) -> TokenStream {
    if opt.debug.is_none() {
        return TokenStream::new();
    }

    let fields = fields.iter().map(|field| {
        let name = field.ident.escaped();
        let opt = &field.opt.debug;

        let value = lift::lift(quote!(self.#name()), (*field.ty).clone())
            .convert_to_native()
            .to_token_stream();

        let value = match &opt.format {
            None => value,
            Some(format) => quote!(format_args!(#format, #value)),
        };

        quote! {
            .field(stringify!(#name), &#value)
        }
    });

    quote! {
        impl ::core::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#ident))
                    #(#fields)*
                    .finish()
            }
        }
    }
}
