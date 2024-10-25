use quote::quote;
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

        if *repr.nonzero {
            quote!(
                unsafe impl ::ribbit::NonZero for #name {}
            )
            .to_tokens(tokens);
        }

        quote!(
            unsafe impl ::ribbit::Pack for #name {
                type Repr = #repr;
            }
        )
        .to_tokens(tokens);
    }
}
