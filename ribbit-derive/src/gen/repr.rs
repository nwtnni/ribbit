use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn repr(
    ir::Struct {
        repr,
        ident,
        vis,
        attrs,
        ..
    }: &ir::Struct,
) -> TokenStream {
    let nonzero = match *repr.nonzero {
        true => quote!(unsafe impl ::ribbit::NonZero for #ident {}),
        false => quote!(),
    };

    let size = repr.size();

    quote! {
        #( #attrs )*
        #vis struct #ident {
            value: #repr,
        }

        unsafe impl ::ribbit::Pack for #ident {
            const BITS: usize = #size;
            type Tight = #repr;
            type Loose = <#repr as ::ribbit::Pack>::Loose;
        }

        #nonzero
    }
}
