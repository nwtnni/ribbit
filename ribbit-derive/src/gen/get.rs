use core::ops::Deref as _;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::Or;

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = ir.tight();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L({
            r#struct.iter().map(move |field| {
                let value = get_field(0, ty_struct, field);
                let vis = field.vis;
                let get = field.ident.escaped();
                let ty = field.ty.packed();

                quote! {
                    #[inline]
                    #vis const fn #get(&self) -> #ty {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        #value
                    }
                }
            })
        }),
        ir::Data::Enum(_) => Or::R(core::iter::empty()),
    }
}

pub(crate) fn get_field(
    shift: usize,
    ty_struct: &crate::ty::Tight,
    field: &ir::Field,
) -> TokenStream {
    lift::Expr::new(quote!(self.value), ty_struct)
        .extract((shift + field.offset) as u8, field.ty.deref())
        .canonicalize()
        .to_token_stream()
}
