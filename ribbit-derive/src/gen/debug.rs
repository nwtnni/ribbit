use darling::FromMeta;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;

#[derive(FromMeta, Debug, Default)]
pub(crate) struct FieldOpt {
    format: Option<syn::LitStr>,
}

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.0.fields.iter().map(|field| {
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

        let ident = self.0.ident;
        quote! {
            impl ::core::fmt::Debug for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.debug_struct(stringify!(#ident))
                        #(#fields)*
                        .finish()
                }
            }
        }
        .to_tokens(tokens)
    }
}
