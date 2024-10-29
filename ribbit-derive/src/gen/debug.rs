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
        fields,
        opt,
        ident,
        generics,
        ..
    }: &ir::Struct,
) -> TokenStream {
    if opt.debug.is_none() {
        return TokenStream::new();
    }

    let fields = fields.iter().map(|field| {
        let name = field.ident.escaped();
        let opt = &field.opt.debug;

        let value = lift::lift(quote!(self.#name()), (*field.ty).clone())
            .ty_to_native()
            .to_token_stream();

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
