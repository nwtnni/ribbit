use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn ord(ir: &ir::Ir) -> TokenStream {
    if ir.opt().ord.is_none() {
        return TokenStream::new();
    }

    let (generics_impl, generics_type, generics_where) = ir.generics().split_for_impl();
    let packed = ir.ident_packed();

    quote! {
        impl #generics_impl PartialOrd for #packed #generics_type #generics_where {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl #generics_impl Ord for #packed #generics_type #generics_where {
            #[inline]
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.value.cmp(&other.value)
            }
        }
    }
}
