use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

pub(crate) fn get<'ir>(
    ir::Struct { fields, repr, .. }: &'ir ir::Struct,
) -> impl Iterator<Item = TokenStream> + 'ir {
    fields.iter().map(|field| {
        let ty_struct = **repr;
        let ty_field = &*field.ty;

        let value_field = lift::lift(quote!(self.value), ty_struct)
            .ty_to_native()
            .apply(lift::Op::Shift {
                dir: lift::Dir::R,
                shift: field.offset,
            })
            .apply(lift::Op::Cast(ty_field.to_native()))
            .apply(lift::Op::And(ty_field.mask()))
            .native_to_ty(ty_field.clone());

        let vis = field.vis;
        let get = field.ident.escaped();
        quote! {
            #[inline]
            #vis const fn #get(&self) -> #ty_field {
                let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                #value_field
            }
        }
    })
}
