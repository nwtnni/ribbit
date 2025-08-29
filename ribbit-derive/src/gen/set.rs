use core::iter;
use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;

pub(crate) fn set<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L({
            r#struct.iter_nonzero().map(
                move |ir::Field {
                          vis,
                          ident,
                          ty,
                          offset,
                          ..
                      }| {
                    let escaped = ident.escaped();
                    let value = lift::Expr::or(
                        ir.r#type(),
                        [
                            (*offset as u8, lift::Expr::new(ident.escaped(), ty.deref())),
                            (
                                0,
                                lift::Expr::new(quote!(self), ir.r#type())
                                    .hole(*offset as u8, ty.deref()),
                            ),
                        ],
                    )
                    .compile();

                    let with = ident.unescaped("with");
                    let ty_field = ty.packed();

                    quote! {
                        #[inline]
                        #vis const fn #with(self, #escaped: #ty_field) -> Self {
                            let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                            #value
                        }
                    }
                },
            )
        }),
        ir::Data::Enum(_) => Or::R(iter::empty()),
    }
}
