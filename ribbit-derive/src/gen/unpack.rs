use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;

pub(crate) fn unpack(ir: &ir::Ir) -> TokenStream {
    let unpacked = ir.ident_unpacked();
    let packed = ir.ident_packed();

    let unpack = match &ir.data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.iter().map(|field| {
                let unescaped = &field.ident;
                let escaped = field.ident.escaped();
                let value = field.r#type.unpack(quote!(self.#escaped()));
                quote!(#unescaped: #value)
            });

            quote! {
                #unpacked {
                    #(#fields ,)*
                }
            }
        }
        ir::Data::Enum(r#enum) => {
            let variants = r#enum.variants.iter().map(|variant| {
                let max_offset = r#enum.discriminant.size
                    + variant
                        .r#struct
                        .fields
                        .iter()
                        .map(|field| field.offset)
                        .max()
                        .unwrap_or(0);

                let fields = variant.r#struct.fields.iter().map(|field| {
                    let name = &field.ident;
                    let value = field.r#type.unpack(crate::gen::get::get_field(
                        &r#enum.r#type,
                        field,
                        max_offset,
                        (r#enum.discriminant.size + field.offset) as u8,
                    ));

                    quote!(#name: #value)
                });

                let discriminant = r#enum
                    .r#type
                    .as_tight()
                    .to_loose()
                    .literal(variant.discriminant as u128);

                let ident = &variant.r#struct.unpacked;

                quote!(#discriminant => #unpacked::#ident { #(#fields ,)* })
            });

            let discriminant = lift::Expr::value_self(&r#enum.r#type)
                .and(r#enum.discriminant.mask)
                .compile(r#enum.r#type.to_loose());

            quote! {
                match #discriminant {
                    #(#variants, )*
                    _ => unsafe {
                        ::core::hint::unreachable_unchecked()
                    }
                }
            }
        }
    };

    let generics = ir.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    let tight = ir.r#type().as_tight();
    let size = tight.size();
    let loose = tight.to_loose();

    quote! {
        unsafe impl #generics_impl ::ribbit::Unpack for #packed #generics_type #generics_where {
            const BITS: usize = #size;
            type Unpacked = #unpacked #generics_type;
            type Loose = #loose;
            #[inline]
            fn unpack(self) -> #unpacked #generics_type {
                #unpack
            }
        }
    }
}
