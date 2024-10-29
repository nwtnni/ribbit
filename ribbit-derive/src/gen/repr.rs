use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn repr(
    ir::Struct {
        repr,
        ident,
        vis,
        attrs,
        generics,
        ..
    }: &ir::Struct,
) -> TokenStream {
    let nonzero = match *repr.nonzero {
        true => quote!(unsafe impl ::ribbit::NonZero for #ident {}),
        false => quote!(),
    };

    let size = repr.size();
    let (r#impl, ty, r#where) = generics.split_for_impl();

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ()));
    let tys = generics.type_params();

    quote! {
        #( #attrs )*
        #vis struct #ident #ty {
            value: #repr,
            r#type: ::ribbit::private::PhantomData<fn(#(#lifetimes),*) -> (#(#tys),*)>,
        }

        unsafe impl #r#impl ::ribbit::Pack for #ident #ty #r#where {
            const BITS: usize = #size;
            type Tight = #repr;
            type Loose = <#repr as ::ribbit::Pack>::Loose;
        }

        #nonzero
    }
}
