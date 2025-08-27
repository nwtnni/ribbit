use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens as _;
use syn::parse_quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt {
    format: Option<syn::LitStr>,
}

pub(crate) fn debug(ir @ ir::Ir { opt, data, .. }: &ir::Ir) -> TokenStream {
    if opt.debug.is_none() {
        return TokenStream::new();
    }

    let packed = ir.ident_packed();
    let unpacked = ir.ident_unpacked();

    // Add Unpacked: Debug clause to where bound
    let mut generics = ir.generics_bounded(None).clone();
    let (_, generics_ty, _) = ir.generics().split_for_impl();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#unpacked #generics_ty: ::core::fmt::Debug));
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    quote! {
        impl #generics_impl ::core::fmt::Debug for #packed #generics_ty #generics_where {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::ribbit::Unpack::unpack(*self).fmt(f)
            }
        }
    }
}
