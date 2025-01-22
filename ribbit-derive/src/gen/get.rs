use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift::Lift as _;
use crate::Or;

pub(crate) fn get<'ir>(
    ir @ ir::Ir {
        ident, tight, data, ..
    }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = **tight;

    match data {
        ir::Data::Struct(ir::Struct { fields }) => Or::L(
            fields
                .iter()
                .filter(|field| *field.ty.size_expected() != 0)
                .map(move |field| {
                    let ty_field = &*field.ty;

                    #[allow(clippy::precedence)]
                    let value_field = ((quote!(self.value).lift() % ty_struct >> field.offset)
                        % ty_field.loosen()
                        & ty_field.mask())
                        % ty_field.clone();

                    let vis = field.vis;
                    let get = field.ident.escaped();
                    quote! {
                        #[inline]
                        #vis const fn #get(&self) -> #ty_field {
                            let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                            #value_field
                        }
                    }
                }),
        ),
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = ir::Enum::unpacked(ident);

            let variants = variants.iter().enumerate().map(|(index, variant)| {
                let discriminant = tight.loosen().literal(index);
                let ident = &variant.ident;
                let value = match variant.ty.as_deref().cloned() {
                    None => quote!(#unpacked::#ident),
                    Some(ty_variant) => {
                        #[allow(clippy::precedence)]
                        let inner = (quote!(self.value).lift() % ty_struct
                            >> r#enum.discriminant_size())
                            % ty_variant;

                        quote!(#unpacked::#ident(#inner))
                    }
                };

                quote!(#discriminant => #value)
            });

            let discriminant = (quote!(self.value).lift() % ty_struct) & r#enum.discriminant_mask();

            let (_, ty, _) = ir.generics().split_for_impl();
            Or::R(std::iter::once(quote! {
                #[inline]
                pub fn unpack(&self) -> #unpacked #ty {
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
