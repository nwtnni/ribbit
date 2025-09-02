use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn hash(ir: &ir::Ir) -> TokenStream {
    if ir.opt().hash.is_none() {
        return TokenStream::new();
    }

    let (generics_impl, generics_type, generics_where) = ir.generics().split_for_impl();
    let packed = ir.ident_packed();

    quote!(
        impl #generics_impl ::core::hash::Hash for #packed #generics_type #generics_where {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.value.hash(state);
            }
        }
    )
}
