use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;

use crate::ir;

pub(crate) fn pre(ir::Struct { fields, .. }: &ir::Struct) -> TokenStream {
    let fields = fields
        .iter()
        .map(|field| &field.ty)
        .filter(|ty| !ty.is_leaf());

    let nonzero = fields
        .clone()
        .filter(|ty| *ty.nonzero())
        .map(|repr| quote!(::ribbit::private::assert_impl_all!(#repr: ::ribbit::NonZero);));

    let pack = fields
        .map(|ty| {
            let size = ty.size();
            quote_spanned! {size.span()=>
                const _: () = if #size != <#ty as ::ribbit::Pack>::BITS {
                    panic!(concat!("Annotated size does not match actual size of type ", stringify!(#ty)));
                };
            }
        });

    quote! {
        #(#pack)*
        #(#nonzero)*
    }
}
