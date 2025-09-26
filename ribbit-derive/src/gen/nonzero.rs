use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn nonzero<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ident = &ir.ident_unpacked();
    let generics = ir.generics();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    ir.opt()
        .nonzero
        .map(|_| quote! {
            #[automatically_derived]
            unsafe impl #generics_impl ::ribbit::NonZero for #ident #generics_type #generics_where {}
        })
        .into_iter()
}
