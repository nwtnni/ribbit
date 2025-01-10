use core::iter;
use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens as _;

use crate::ir;
use crate::lift::Lift as _;
use crate::ty;
use crate::Or;

pub(crate) fn set<'ir>(
    ir::Ir { tight, data, .. }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    match data {
        ir::Data::Struct(ir::Struct { fields }) => Or::L(
            fields
                .iter()
                .filter(|field| *field.ty.size() != 0)
                .map(
                    |ir::Field {
                         vis,
                         ident,
                         ty,
                         offset,
                         ..
                     }| { (vis, ident, ty.deref().clone(), *offset) },
                )
                .map(|(vis, ident, ty_field, offset)| {
                    let ty_struct = ty::Tree::from(**tight);
                    let ty_struct_loose = tight.loosen();

                    let escaped = ident.escaped();

                    #[allow(clippy::precedence)]
                    let value_field = Box::new(
                        escaped.to_token_stream().lift() % ty_field.clone() % ty_struct_loose
                            << offset,
                    );

                    let clear = !(ty_field.mask() << offset) & ty_struct.mask();

                    #[allow(clippy::precedence)]
                    let value_struct = (quote!(self.value).lift() % ty_struct.clone() & clear
                        | value_field)
                        % ty_struct;

                    let with = ident.unescaped("with");
                    quote! {
                        #[inline]
                        #vis const fn #with(&self, #escaped: #ty_field) -> Self {
                            let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                            Self {
                                value: #value_struct,
                                r#type: ::ribbit::private::PhantomData,
                            }
                        }
                    }
                }),
        ),
        ir::Data::Enum(_) => Or::R(iter::empty()),
    }
}
