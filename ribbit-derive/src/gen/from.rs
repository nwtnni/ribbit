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

    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();
    let packed = ir.ident_packed();
    let unpacked = ir.ident_unpacked();

    quote! {
        impl #generics_impl From<#unpacked #generics_ty> for #packed #generics_ty #generics_where {
            #[inline]
            fn from(unpacked: #unpacked #generics_ty) -> Self {
                unpacked.pack()
            }
        }

        impl #generics_impl From<#packed #generics_ty> for #unpacked #generics_ty #generics_where {
            #[inline]
            fn from(packed: #packed #generics_ty) -> Self {
                packed.unpack()
            }
        }
    }
}
