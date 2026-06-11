use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt(ir::CommonOpt);

pub(crate) fn from_raw_unchecked<'ir>(ir: &'ir ir::Ir) -> TokenStream {
    let opt = &ir.opt().from_raw_unchecked.0;

    if opt.skip {
        return TokenStream::default();
    }

    let vis = opt.vis(&ir.vis);
    let from_raw_unchecked = opt.rename_with(|| Cow::Owned(format_ident!("from_raw_unchecked")));
    let tight = ir.r#type().as_tight();
    let precondition = crate::gen::precondition::assert();

    quote! {
        #[inline]
        #vis const unsafe fn #from_raw_unchecked(value: #tight) -> Self {
            #precondition
            Self {
                value,
                r#type: ::ribbit::PhantomData,
            }
        }
    }
}
