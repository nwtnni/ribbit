use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;

pub(crate) fn unpack(item: &ir::Item) -> TokenStream {
    let unpacked = item.ident_unpacked();
    let packed = item.ident_packed();

    let unpack = match &item.data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.iter().map(|field| {
                let unescaped = &field.ident;
                let value = field.r#type.unpack(crate::gen::get::get_field(
                    &r#struct.r#type,
                    field,
                    r#struct.max_offset,
                    field.offset as u8,
                ));
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
                let fields = variant.r#struct.fields.iter().map(|field| {
                    let name = &field.ident;
                    let value = field.r#type.unpack(crate::gen::get::get_field(
                        &r#enum.r#type,
                        field,
                        r#enum.discriminant.size + variant.r#struct.max_offset,
                        (r#enum.discriminant.size + field.offset) as u8,
                    ));

                    quote!(#name: #value)
                });

                let discriminant = r#enum
                    .r#type
                    .as_tight()
                    .to_loose()
                    .literal(variant.discriminant as u128);

                let ident = &variant.ident;

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

    let generics = item.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    let tight = item.r#type().as_tight();
    let size = tight.size();
    let loose = tight.to_loose();

    quote! {
        unsafe impl #generics_impl ::ribbit::Unpack for #packed #generics_type #generics_where {
            const BITS: usize = #size;

            type Unpacked = #unpacked #generics_type;
            type Loose = #loose;
            type Raw = #tight;

            #[inline]
            fn unpack(self) -> #unpacked #generics_type {
                #unpack
            }

            #[inline]
            fn into_raw(self) -> Self::Raw {
                self.value
            }

            #[inline]
            unsafe fn from_raw_unchecked(raw: Self::Raw) -> Self {
                Self {
                    value: raw,
                    r#type: ::ribbit::PhantomData,
                }
            }
        }
    }
}
