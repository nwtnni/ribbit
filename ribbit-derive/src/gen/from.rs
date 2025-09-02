use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn from(ir: &ir::Ir) -> TokenStream {
    let Some(StructOpt) = &ir.opt().from else {
        return TokenStream::new();
    };

    let generics = ir.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();
    let packed = ir.ident_packed();
    let unpacked = ir.ident_unpacked();

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
