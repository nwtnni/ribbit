use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.0.fields.iter().map(|field| {
            let ty_field = &*field.ty;
            let ty_struct = *self.0.repr;

            // Shift field by offset
            let ident = field.ident.escaped();
            let value_field = lift::lift(&ident, ty_field.clone())
                .convert_to_native()
                .apply(lift::Op::Cast(ty_struct.to_native()))
                .apply(lift::Op::Shift {
                    dir: lift::Dir::L,
                    shift: field.offset,
                });

            let value_struct = lift::lift(quote!(self.value), ty_struct)
                .convert_to_native()
                // Clear hole in struct
                .apply(lift::Op::And(
                    !(ty_field.mask() << field.offset) & ty_struct.mask(),
                ))
                .apply(lift::Op::Or(Box::new(value_field)))
                .convert_to_ty(ty_struct);

            let vis = field.vis;
            let with = field.ident.unescaped("with");
            quote! {
                #vis const fn #with(&self, #ident: #ty_field) -> Self {
                    Self {
                        value: #value_struct,
                    }
                }
            }
        });

        let ident = self.0.ident;
        quote! {
            impl #ident {
                #( #fields )*
            }
        }
        .to_tokens(tokens)
    }
}
