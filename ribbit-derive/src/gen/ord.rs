use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn ord(ir @ ir::Ir { opt, ident, .. }: &ir::Ir) -> TokenStream {
    if opt.ord.is_none() {
        return TokenStream::new();
    }

    let (r#impl, ty, r#where) = ir.generics().split_for_impl();

    quote! {
        impl #r#impl PartialOrd for #ident #ty #r#where {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl #r#impl Ord for #ident #ty #r#where {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.value.cmp(&other.value)
            }
        }
    }
}
