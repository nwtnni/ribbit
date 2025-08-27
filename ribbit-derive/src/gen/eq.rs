use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn eq(ir: &ir::Ir) -> TokenStream {
    if ir.opt.eq.is_none() {
        return TokenStream::new();
    }

    let (r#impl, ty, r#where) = ir.generics().split_for_impl();
    let ident = ir.ident_packed();

    quote!(
        impl #r#impl Eq for #ident #ty #r#where {}
        impl #r#impl PartialEq for #ident #ty #r#where {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.value.eq(&other.value)
            }
        }
    )
}
