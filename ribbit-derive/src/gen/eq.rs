use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct ItemOpt;

pub(crate) fn eq(item: &ir::Item) -> TokenStream {
    if item.opt().derive.eq.is_none() {
        return TokenStream::new();
    }

    let (generics_impl, generics_type, generics_where) = item.generics().split_for_impl();
    let packed = item.ident_packed();

    quote!(
        impl #generics_impl Eq for #packed #generics_type #generics_where {}

        impl #generics_impl PartialEq for #packed #generics_type #generics_where {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.value.eq(&other.value)
            }
        }
    )
}
