use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn copy(
    ir::Ir {
        opt,
        ident,
        generics,
        ..
    }: &ir::Ir,
) -> TokenStream {
    if opt.copy.is_none() {
        return TokenStream::new();
    }

    let (r#impl, ty, r#where) = generics.split_for_impl();

    quote!(
        impl #r#impl Copy for #ident #ty #r#where {}
        impl #r#impl Clone for #ident #ty #r#where {
            fn clone(&self) -> Self {
                *self
            }
        }
    )
}
