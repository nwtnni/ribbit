use core::iter;
use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;

pub(crate) fn set<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ir::Data::Struct(r#struct) = &ir.data else {
        return Or::L(iter::empty());
    };

    Or::R(r#struct.iter_nonzero().map(
        move |ir::Field {
                  vis,
                  ident,
                  ty,
                  offset,
                  ..
              }| {
            let escaped = ident.escaped();
            let value = lift::Expr::or([
                lift::Expr::value(ident.escaped(), ty.deref()).shift_left(*offset as u8),
                lift::Expr::value_self(&r#struct.r#type)
                    .and(!(ty.mask() << *offset) & ir.r#type().as_tight().mask()),
            ])
            .compile(ir.r#type().as_tight());

            let with = ident.unescaped("with");
            let ty_field = ty.packed();
            let precondition = crate::gen::pre::precondition();

            quote! {
                #[inline]
                #vis const fn #with(self, #escaped: #ty_field) -> Self {
                    #precondition
                    Self {
                        value: #value,
                        r#type: ::ribbit::private::PhantomData,
                    }
                }
            }
        },
    ))
}
