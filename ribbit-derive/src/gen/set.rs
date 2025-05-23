use core::iter;
use core::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift::Lift as _;
use crate::ty;
use crate::Or;

pub(crate) fn set<'ir>(
    ir::Ir { tight, data, .. }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    match data {
        ir::Data::Struct(r#struct) => Or::L({
            let newtype = r#struct.is_newtype();

            r#struct
                .iter_nonzero()
                .map(
                    |ir::Field {
                         vis,
                         ident,
                         ty,
                         offset,
                         ..
                     }| { (vis, ident, ty.deref().clone(), *offset) },
                )
                .map(move |(vis, ident, ty_field, offset)| {
                    let ty_struct = ty::Tree::from(tight.clone());
                    let ty_struct_loose = tight.loosen();

                    let escaped = ident.escaped().to_token_stream();

                    let value_struct = match newtype {
                        true if ty_field.is_leaf() => escaped.clone(),
                        true => (escaped.clone().lift() % ty_field.clone() % ty_struct)
                            .to_token_stream(),
                        #[allow(clippy::precedence)]
                        false => {
                            let value_field = Box::new(
                                escaped.to_token_stream().lift()
                                    % ty_field.clone()
                                    % ty_struct_loose
                                    << offset,
                            );

                            let clear = !(ty_field.mask() << offset) & ty_struct.mask();

                            (quote!(self.value).lift() % ty_struct.clone() & clear | value_field)
                                % ty_struct
                        }
                        .to_token_stream(),
                    };

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
                })
        }),
        ir::Data::Enum(_) => Or::R(iter::empty()),
    }
}
