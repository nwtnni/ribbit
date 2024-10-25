use quote::quote;
use quote::ToTokens;

use crate::ir;

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.0.fields.iter().map(|field| {
            let name = field.ident.escaped();
            quote! {
                .field(stringify!(#name), &self.#name())
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
