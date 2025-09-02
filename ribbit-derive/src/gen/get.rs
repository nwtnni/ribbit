use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::r#type::Type;
use crate::Or;

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ir::Data::Struct(r#struct) = &ir.data else {
        return Or::L(core::iter::empty());
    };

    let precondition = crate::gen::pre::precondition();

    Or::R({
        r#struct.iter().map(move |field| {
            let value = get_field(&r#struct.r#type, field, field.offset as u8);
            let vis = field.vis;
            let get = field.ident.escaped();
            let r#type = field.r#type.packed();

            quote! {
                #[inline]
                #vis const fn #get(self) -> #r#type {
                    #precondition
                    #value
                }
            }
        })
    })
}

pub(crate) fn get_field(r#type: &Type, field: &ir::Field, offset: u8) -> TokenStream {
    let expr = lift::Expr::value_self(r#type).shift_right(offset);

    match field.r#type.is_loose() {
        true => expr,
        false => expr.and(field.r#type.mask()),
    }
    .compile(&*field.r#type)
}
