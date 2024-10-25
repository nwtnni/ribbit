use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident;

        let parameters = self.0.fields.iter().map(|field| {
            let ident = field.ident.escaped();
            let repr = &field.ty;
            quote!(#ident: #repr)
        });

        let value_struct = self
            .0
            .fields
            .iter()
            .fold(
                Box::new(lift::zero(self.0.repr.to_native())) as Box<dyn lift::Native>,
                |state, field| {
                    let ident = field.ident.escaped();
                    let value_field = lift::lift(ident, (*field.ty).clone())
                        .convert_to_native()
                        .apply(lift::Op::Cast(self.0.repr.to_native()))
                        .apply(lift::Op::Shift {
                            dir: lift::Dir::L,
                            shift: field.offset,
                        });

                    Box::new(state.apply(lift::Op::Or(Box::new(value_field))))
                },
            )
            .convert_to_ty(*self.0.repr);

        quote! {
            impl #ident {
                pub const fn new(
                    #(#parameters),*
                ) -> Self {
                    Self { value: #value_struct }
                }
            }
        }
        .to_tokens(tokens)
    }
}
