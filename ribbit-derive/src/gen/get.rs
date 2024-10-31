use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;
use crate::Or;

pub(crate) fn get<'ir>(
    ir::Ir {
        ident, repr, data, ..
    }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    match data {
        ir::Data::Struct(ir::Struct { fields }) => Or::L(fields.iter().map(|field| {
            let ty_struct = **repr;
            let ty_field = &*field.ty;

            let value_field = lift::lift(quote!(self.value), ty_struct)
                .ty_to_native()
                .apply(lift::Op::Shift {
                    dir: lift::Dir::R,
                    shift: field.offset,
                })
                .apply(lift::Op::Cast(ty_field.to_native()))
                .apply(lift::Op::And(ty_field.mask()))
                .native_to_ty(ty_field.clone());

            let vis = field.vis;
            let get = field.ident.escaped();
            quote! {
                #[inline]
                #vis const fn #get(&self) -> #ty_field {
                    let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                    #value_field
                }
            }
        })),
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = r#enum.unpacked(ident);

            let variants = variants.iter().enumerate().map(|(index, variant)| {
                let discriminant = repr.to_native().literal(index);
                let ident = &variant.ident;
                let value = match &variant.ty {
                    None => quote!(#unpacked::#ident),
                    Some(ty) => {
                        let inner = lift::lift(quote!(self.value), repr.to_native())
                            .ty_to_native()
                            .apply(lift::Op::Shift {
                                dir: lift::Dir::R,
                                shift: r#enum.discriminant_size(),
                            })
                            .native_to_ty((**ty).clone());

                        quote!(#unpacked::#ident(#inner))
                    }
                };

                quote!(#discriminant => #value)
            });

            let discriminant = lift::lift(quote!(self.value), repr.to_native())
                .ty_to_native()
                .apply(lift::Op::And(r#enum.discriminant_mask()));

            Or::R(std::iter::once(quote! {
                #[inline]
                pub fn unpack(&self) -> #unpacked {
                    match #discriminant {
                        #(#variants,)*
                        _ => unsafe {
                            ::core::hint::unreachable_unchecked()
                        }
                    }
                }
            }))
        }
    }
}
