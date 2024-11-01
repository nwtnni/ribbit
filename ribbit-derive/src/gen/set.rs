use core::iter;

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
        ir::Data::Struct(ir::Struct { fields }) => Or::L(fields.iter().map(|field| {
            let ty_field = &*field.ty;
            let ty_struct = **tight;

            let ident = field.ident.escaped().to_token_stream();
            let value_field =
                (ident.clone().lift() % ty_field.clone() % ty_struct.loosen()) << field.offset;

            let clear = !(ty_field.mask() << field.offset) & ty_struct.mask();
            let value_struct = ((quote!(self.value).lift() % ty_struct) & clear
                | Box::new(value_field))
                % ty::Tree::from(ty_struct);

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
        })),
        ir::Data::Enum(_) => Or::R(iter::empty()),
    }
}
