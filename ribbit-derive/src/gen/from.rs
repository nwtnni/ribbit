use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct ItemOpt;

pub(crate) fn from(item: &ir::Item) -> TokenStream {
    let Some(ItemOpt) = &item.opt().derive.from else {
        return TokenStream::new();
    };

    let generics = item.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();
    let packed = item.ident_packed();
    let unpacked = item.ident_unpacked();

    quote! {
        impl #generics_impl From<#unpacked #generics_type> for #packed #generics_type #generics_where {
            #[inline]
            fn from(unpacked: #unpacked #generics_type) -> Self {
                unpacked.pack()
            }
        }

        impl #generics_impl From<#packed #generics_type> for #unpacked #generics_type #generics_where {
            #[inline]
            fn from(packed: #packed #generics_type) -> Self {
                packed.unpack()
            }
        }
    }
}
