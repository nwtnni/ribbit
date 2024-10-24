use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::repr::Leaf;
use crate::Spanned;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> Self {
        Self(r#struct)
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident();
        let parameters = self.0.fields().iter().map(Parameter);
        let arguments = self.0.fields().iter().map(|field| Argument {
            repr: self.0.repr(),
            field,
        });

        let value = self.0.repr().convert_from_native(quote!(#((#arguments))|*));

        quote! {
            impl #ident {
                pub const fn new(
                    #(#parameters),*
                ) -> Self {
                    Self { value: #value }
                }
            }
        }
        .to_tokens(tokens)
    }
}

struct Parameter<'ir>(&'ir ir::Field<'ir>);

impl ToTokens for Parameter<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident().expect("Field names required");
        let repr = self.0.repr();
        quote!(#ident: #repr).to_tokens(tokens);
    }
}

struct Argument<'ir> {
    repr: &'ir Spanned<Leaf>,
    field: &'ir ir::Field<'ir>,
}

impl ToTokens for Argument<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.field.ident().expect("Field names required");
        let offset = self.field.offset();
        let repr = self.field.repr();

        let small = repr.convert_to_native(ident);
        self.repr
            .as_native()
            .to_native(quote!((#small << #offset)))
            .to_tokens(tokens)
    }
}
