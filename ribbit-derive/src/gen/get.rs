use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) struct Struct<'ir>(pub(crate) &'ir ir::Struct<'ir>);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.0.fields.iter().map(|field| {
            let ty_struct = *self.0.repr;
            let ty_field = &*field.ty;

            let value_field = lift::lift(quote!(self.value), ty_struct)
                .convert_to_native()
                .apply(lift::Op::Shift {
                    dir: lift::Dir::R,
                    shift: field.offset,
                })
                .convert_to_ty(ty_field.clone());

            let vis = field.vis;
            let get = field.ident.escaped();
            quote! {
                #vis const fn #get(&self) -> #ty_field {
                    #value_field
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
