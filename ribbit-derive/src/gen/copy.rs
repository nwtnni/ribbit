use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn copy(ir @ ir::Ir { item, .. }: &ir::Ir) -> TokenStream {
    let (r#impl, ty, r#where) = ir.generics().split_for_impl();
    let ident = &item.ident;

    quote!(
        impl #r#impl Copy for #ident #ty #r#where {}
        impl #r#impl Clone for #ident #ty #r#where {
            #[inline]
            fn clone(&self) -> Self {
                *self
            }
        }
    )
}
