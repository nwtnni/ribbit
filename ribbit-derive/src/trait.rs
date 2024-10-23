use quote::quote;
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
        if self.0.repr().nonzero {
            if self.0.fields().iter().all(|field| !field.nonzero()) {
                panic!("At least one field must be non-zero")
            }

            let ident = self.0.ident();

            quote!(
                unsafe impl ::ribbit::NonZero for #ident {}
            )
            .to_tokens(tokens);
        }
    }
}
