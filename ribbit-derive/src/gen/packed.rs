use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn packed(ir: &ir::Ir) -> TokenStream {
    let vis = &ir.vis;
    let tight = ir.r#type().as_tight();

    let generics = ir.generics();
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ()));
    let tys = generics.type_params();
    let packed = ir.ident_packed();

    quote! {
        #[repr(transparent)]
        #vis struct #packed #generics_ty {
            value: #tight,
            r#type: ::ribbit::private::PhantomData<fn(#(#lifetimes),*) -> (#(#tys),*)>,
        }

        #[automatically_derived]
        impl #generics_impl Copy for #packed #generics_ty #generics_where {}

        #[automatically_derived]
        impl #generics_impl Clone for #packed #generics_ty #generics_where {
            fn clone(&self) -> Self {
                *self
            }
        }
    }
}
