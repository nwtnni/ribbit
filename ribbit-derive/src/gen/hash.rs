use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn hash(ir @ ir::Ir { item, .. }: &ir::Ir) -> TokenStream {
    if item.opt.hash.is_none() {
        return TokenStream::new();
    }

    let (r#impl, ty, r#where) = ir.generics().split_for_impl();
    let ident = &item.ident;

    quote!(
        impl #r#impl ::core::hash::Hash for #ident #ty #r#where {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.value.hash(state);
            }
        }
    )
}
