use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt(ir::CommonOpt);

pub(crate) fn into_raw<'ir>(ir: &'ir ir::Ir) -> TokenStream {
    let opt = &ir.opt().into_raw.0;

    if opt.skip {
        return TokenStream::default();
    }

    let vis = opt.vis(&ir.vis);
    let into_raw = opt.rename_with(|| Cow::Owned(format_ident!("into_raw")));
    let tight = ir.r#type().as_tight();
    let precondition = crate::gen::precondition::assert();

    quote! {
        #[inline]
        #vis const fn #into_raw(self) -> #tight {
            #precondition
            self.value
        }
    }
}
