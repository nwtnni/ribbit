use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) fn get(
    ir::Struct {
        fields,
        repr,
        ident,
        ..
    }: &ir::Struct,
) -> TokenStream {
    let fields = fields.iter().map(|field| {
        let ty_struct = **repr;
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
            #[inline]
            #vis const fn #get(&self) -> #ty_field {
                #value_field
            }
        }
    });

    quote! {
        impl #ident {
            #(#fields)*
        }
    }
}
