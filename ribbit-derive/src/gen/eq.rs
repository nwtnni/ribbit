use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn eq(ir @ ir::Ir { opt, ident, .. }: &ir::Ir) -> TokenStream {
    if opt.eq.is_none() {
        return TokenStream::new();
    }

    let (r#impl, ty, r#where) = ir.generics().split_for_impl();

    quote!(
        impl #r#impl Eq for #ident #ty #r#where {}
        impl #r#impl PartialEq for #ident #ty #r#where {
            fn eq(&self, other: &Self) -> bool {
                self.value.eq(&other.value)
            }
        }
    )
}
