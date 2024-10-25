use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;

use crate::ir;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> Self {
        Self(r#struct)
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let nonzero = self
            .0
            .fields
            .iter()
            .filter(|field| *field.ty.nonzero())
            .map(|field| &field.ty)
            .map(|repr| quote!(::ribbit::private::assert_impl_all!(#repr: ::ribbit::NonZero);));

        let pack = self
            .0
            .fields
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
        .to_tokens(tokens)
    }
}
