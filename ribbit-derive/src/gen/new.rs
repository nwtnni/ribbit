use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> Self {
        Self(r#struct)
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident;

        let parameters = self.0.fields.iter().map(|field| {
            let ident = field.ident.escaped();
            let repr = field.repr;
            quote!(#ident: #repr)
        });

        let value = self
            .0
            .fields
            .iter()
            .fold(
                Box::new(lift::zero(self.0.repr.as_native())) as Box<dyn lift::Native>,
                |state, field| {
                    let ident = field.ident.escaped();
                    let value = lift::lift(ident, *field.repr)
                        .into_native()
                        .apply(lift::Op::Cast(self.0.repr.as_native()))
                        .apply(lift::Op::Shift {
                            dir: lift::Dir::L,
                            shift: field.offset(),
                        });

                    Box::new(state.apply(lift::Op::Or(Box::new(value))))
                },
            )
            .into_repr((*self.0.repr).into());

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
