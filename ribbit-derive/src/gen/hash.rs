use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct ItemOpt;

pub(crate) fn hash(item: &ir::Item) -> TokenStream {
    if item.opt().derive.hash.is_none() {
        return TokenStream::new();
    }

    let (generics_impl, generics_type, generics_where) = item.generics().split_for_impl();
    let packed = item.ident_packed();

    quote!(
        impl #generics_impl ::core::hash::Hash for #packed #generics_type #generics_where {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.value.hash(state);
            }
        }
    )
}
