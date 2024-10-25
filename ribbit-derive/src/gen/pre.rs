use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;

use crate::ir;

pub(crate) fn pre(ir::Struct { fields, .. }: &ir::Struct) -> TokenStream {
    let nonzero = fields
        .iter()
        .filter(|field| *field.ty.nonzero())
        .map(|field| &field.ty)
        .map(|repr| quote!(::ribbit::private::assert_impl_all!(#repr: ::ribbit::NonZero);));

    let pack = fields
        .iter()
        .map(|field| {
            let repr = &field.ty;
            let size = repr.size();
            quote_spanned! {size.span()=>
                const _: () = if #size != <<#repr as ::ribbit::Pack>::Repr as ::ribbit::Number>::BITS {
                    panic!(concat!("Annotated size does not match actual size of type ", stringify!(#repr)));
                };
            }
        });

    quote! {
        #(#pack)*
        #(#nonzero)*
    }
}
