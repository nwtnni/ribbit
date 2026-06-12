use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt(ir::CommonOpt);

pub(crate) fn into_raw(ir: &ir::Ir) -> TokenStream {
    let opt = &ir.opt().into_raw;

    if opt.0.skip {
        return TokenStream::default();
    }

    let vis = opt.0.vis(&ir.vis);
    let name = opt.0.rename_with(|| Cow::Owned(format_ident!("into_raw")));
    let tight = ir.r#type().as_tight();
    let precondition = crate::gen::precondition::assert();

    quote! {
        #[inline]
        #vis const fn #name(self) -> #tight {
            #precondition
            self.value
        }
    }
}
