use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct ItemOpt;

pub(crate) fn debug(item: &ir::Item) -> TokenStream {
    if item.opt().derive.debug.is_none() {
        return TokenStream::new();
    }

    let packed = item.ident_packed();
    let unpacked = item.ident_unpacked();

    // Add Unpacked: Debug clause to where bound
    let mut generics = item.generics_bounded().clone();
    let (_, generics_type, _) = item.generics().split_for_impl();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#unpacked #generics_type: ::core::fmt::Debug));
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    quote! {
        impl #generics_impl ::core::fmt::Debug for #packed #generics_type #generics_where {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::ribbit::Unpack::unpack(*self).fmt(f)
            }
        }
    }
}
