use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;

use crate::error::bail;
use crate::ir;
use crate::Error;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> darling::Result<Self> {
        let repr = r#struct.repr();

        if *repr.nonzero && r#struct.fields().iter().all(|field| !*field.nonzero()) {
            bail!(repr.nonzero=> Error::StructNonZero);
        }

        Ok(Self(r#struct))
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let repr = self.0.repr();
        let name = self.0.ident();

        let nonzero_struct = match *repr.nonzero {
            true => quote!(unsafe impl ::ribbit::NonZero for #name {}),
            false => quote!(),
        };

        let nonzero_fields = self
            .0
            .fields()
            .iter()
            .filter(|field| *field.nonzero())
            .map(ir::Field::repr)
            .map(|repr| quote!(::ribbit::private::assert_impl_all!(#repr: ::ribbit::NonZero);));

        let pack_struct = quote!(unsafe impl ::ribbit::Pack for #name { type Repr = #repr; });

        let pack_fields = self
            .0
            .fields()
            .iter()
            .map(|field| {
                let repr = field.repr();
                let size = field.size();
                quote_spanned! {size.span()=>
                    const _: () = if #size != <<#repr as ::ribbit::Pack>::Repr as ::ribbit::Number>::BITS {
                        panic!(concat!("Annotated size does not match actual size of type ", stringify!(#repr)));
                    };
                }
            });

        quote! {
            #nonzero_struct
            #(#nonzero_fields)*

            #pack_struct
            #(#pack_fields)*
        }
        .to_tokens(tokens)
    }
}
