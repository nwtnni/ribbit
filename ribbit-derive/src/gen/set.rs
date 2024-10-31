use core::iter;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::LoosenExt as _;
use crate::Or;

pub(crate) fn set<'ir>(
    ir::Ir { tight, data, .. }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            Or::L(fields.iter().map(|field| {
                let ty_field = &*field.ty;
                let ty_struct = **tight;

                // Shift field by offset
                let ident = field.ident.escaped();
                let value_field = lift::lift(&ident, ty_field.clone())
                    .apply(lift::Op::Cast(ty_struct.loosen()))
                    .apply(lift::Op::Shift {
                        dir: lift::Dir::L,
                        shift: field.offset,
                    });

                let value_struct = lift::lift(quote!(self.value), ty_struct)
                    // Clear hole in struct
                    .apply(lift::Op::And(
                        !(ty_field.mask() << field.offset) & ty_struct.mask(),
                    ))
                    .apply(lift::Op::Or(Box::new(value_field)))
                    .tighten(ty_struct);

                let vis = field.vis;
                let with = field.ident.unescaped("with");
                quote! {
                    #[inline]
                    #vis const fn #with(&self, #ident: #ty_field) -> Self {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        Self {
                            value: #value_struct,
                            r#type: ::ribbit::private::PhantomData,
                        }
                    }
                }
            }))
        }
        ir::Data::Enum(_) => Or::R(iter::empty()),
    }
}
