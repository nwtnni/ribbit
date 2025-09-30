use core::iter;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;

pub(crate) fn with<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ir::Data::Struct(r#struct) = &ir.data else {
        return Or::L(iter::empty());
    };

    Or::R(r#struct.iter_nonzero().map(move |field| {
        let value = lift::Expr::or([
            lift::Expr::value(field.ident.escaped(), &field.r#type).shift_left(field.offset as u8),
            lift::Expr::value_self(&r#struct.r#type)
                .and(!(field.r#type.mask() << field.offset) & r#struct.r#type.as_tight().mask()),
        ])
        .compile(ir.r#type().as_tight());

        let vis = &field.vis;
        let with = field.ident.unescaped("with");
        let name = field.ident.escaped();
        let r#type = field.r#type.packed();
        let precondition = crate::gen::pre::precondition();

        quote! {
            #[inline]
            #vis const fn #with(self, #name: #r#type) -> Self {
                #precondition
                Self {
                    value: #value,
                    r#type: ::ribbit::private::PhantomData,
                }
            }
        }
    }))
}
