use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift::Lift as _;
use crate::Or;

pub(crate) fn get<'ir>(
    ir @ ir::Ir {
        item, tight, data, ..
    }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = tight;

    match data {
        ir::Data::Struct(r#struct) => Or::L({
            let newtype = r#struct.is_newtype();
            r#struct.iter().map(move |field| {
                let ty_field = &*field.ty;

                let value = quote!(self.value);

                let value_field = match newtype {
                    // Forward underlying type directly
                    true if ty_field.is_leaf() => value.to_token_stream(),
                    // Skip conversion through loose types
                    true => (value.lift() % ty_struct.clone() % ty_field.clone()).to_token_stream(),
                    #[allow(clippy::precedence)]
                    false => {
                        ((value.lift() % ty_struct.clone() >> field.offset) % ty_field.loosen()
                            & ty_field.mask())
                            % ty_field.clone()
                    }
                    .to_token_stream(),
                };

                let vis = field.vis;
                let get = field.ident.escaped();
                quote! {
                    #[inline]
                    #vis const fn #get(&self) -> #ty_field {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        #value_field
                    }
                }
            })
        }),
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = ir::Enum::unpacked(&item.ident);

            let variants = variants.iter().enumerate().map(|(index, variant)| {
                let discriminant = tight.loosen().literal(index as u128);
                let ident = &variant.ident;
                let value = match variant.ty.as_deref().cloned() {
                    None => quote!(#unpacked::#ident),
                    Some(ty_variant) => {
                        #[allow(clippy::precedence)]
                        let inner = (quote!(self.value).lift() % ty_struct.clone()
                            >> r#enum.discriminant_size())
                            % ty_variant;

                        quote!(#unpacked::#ident(#inner))
                    }
                };

                quote!(#discriminant => #value)
            });

            let discriminant =
                (quote!(self.value).lift() % ty_struct.clone()) & r#enum.discriminant_mask();

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
