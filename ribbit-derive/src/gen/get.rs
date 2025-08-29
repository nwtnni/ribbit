use core::ops::Deref as _;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::ty::Type;
use crate::Or;

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = ir.r#type();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L({
            r#struct.iter().map(move |field| {
                let value = get_field(ty_struct, field, field.offset as u8);
                let vis = field.vis;
                let get = field.ident.escaped();
                let ty = field.ty.packed();

                quote! {
                    #[inline]
                    #vis const fn #get(self) -> #ty {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        #value
                    }
                }
            })
        }),
        ir::Data::Enum(_) => Or::R(core::iter::empty()),
    }
}

pub(crate) fn get_field(ty_struct: &Type, field: &ir::Field, offset: u8) -> TokenStream {
    lift::Expr::new(quote!(self), ty_struct)
        .extract(offset, field.ty.deref())
        .compile()
}
