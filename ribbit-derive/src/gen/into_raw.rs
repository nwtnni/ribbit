use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct ItemOpt(ir::CommonOpt);

pub(crate) fn into_raw(item: &ir::Item) -> TokenStream {
    let opt = &item.opt().into_raw;

    if opt.0.skip {
        return TokenStream::default();
    }

    let vis = opt.0.vis(&item.vis);
    let name = opt.0.rename_with(|| Cow::Owned(format_ident!("into_raw")));
    let tight = item.r#type().as_tight();
    let precondition = crate::gen::precondition::assert();

    quote! {
        #[inline]
        #vis const fn #name(self) -> #tight {
            #precondition
            self.value
        }
    }
}
