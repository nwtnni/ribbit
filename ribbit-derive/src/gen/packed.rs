use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn packed(ir: &ir::Ir) -> TokenStream {
    let vis = &ir.vis;
    let packed = ir.ident_packed();
    let tight = ir.r#type().as_tight();

    let generics = ir.generics();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes();
    let types = generics.type_params();

    quote! {
        #[repr(transparent)]
        #vis struct #packed #generics_type {
            value: #tight,
            r#type: ::ribbit::private::PhantomData<fn(#(&#lifetimes ()),*) -> (#(#types),*)>,
        }

        #[automatically_derived]
        impl #generics_impl Copy for #packed #generics_type #generics_where {}

        #[automatically_derived]
        impl #generics_impl Clone for #packed #generics_type #generics_where {
            fn clone(&self) -> Self {
                *self
            }
        }
    }
}
