use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = &self.0.repr;
        let ident = self.0.ident;
        let vis = self.0.vis;
        let attrs = self.0.attrs;

        let nonzero = match *repr.nonzero {
            true => quote!(unsafe impl ::ribbit::NonZero for #ident {}),
            false => quote!(),
        };

        quote! {
            #( #attrs )*
            #vis struct #ident {
                value: #repr,
            }

            unsafe impl ::ribbit::Pack for #ident { type Repr = #repr; }

            #nonzero
        }
        .to_tokens(tokens)
    }
}
