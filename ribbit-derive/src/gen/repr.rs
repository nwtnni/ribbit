use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn repr(
    ir::Ir {
        tight: repr,
        ident,
        vis,
        attrs,
        generics,
        data,
        ..
    }: &ir::Ir,
) -> TokenStream {
    let size = repr.size();
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    let nonzero = match *repr.nonzero {
        true => {
            quote!(unsafe impl #generics_impl ::ribbit::NonZero for #ident #generics_ty #generics_where {})
        }
        false => quote!(),
    };

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ()));
    let tys = generics.type_params();

    let r#struct = quote! {
        #vis struct #ident #generics_ty {
            value: #repr,
            r#type: ::ribbit::private::PhantomData<fn(#(#lifetimes),*) -> (#(#tys),*)>,
        }
    };

    let unpacked = match data {
        ir::Data::Struct(_) => TokenStream::new(),
        ir::Data::Enum(r#enum) => {
            let variants = r#enum
                .variants
                .iter()
                .map(|ir::Variant { ident, ty }| match ty {
                    None => quote!(#ident),
                    Some(ty) => quote!(#ident(#ty)),
                });

            let unpacked = r#enum.unpacked(ident);
            quote! {
                #vis enum #unpacked #generics_ty {
                    #(#variants),*
                }
            }
        }
    };

    quote! {
        #(#attrs)*
        #r#struct

        #unpacked

        unsafe impl #generics_impl ::ribbit::Pack for #ident #generics_ty #generics_where {
            const BITS: usize = #size;
            type Tight = #repr;
            type Loose = <#repr as ::ribbit::Pack>::Loose;
        }

        #nonzero
    }
}
