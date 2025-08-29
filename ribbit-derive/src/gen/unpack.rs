use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;

pub(crate) fn unpack(ir: &ir::Ir) -> TokenStream {
    let ty_struct = ir.r#type();

    let unpacked = ir.ident_unpacked();
    let packed = ir.ident_packed();

    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    let unpack = match &ir.data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.iter().map(|field| {
                let unescaped = &field.ident;
                let escaped = field.ident.escaped();
                let value = field.ty.unpack(quote!(self.#escaped()));
                quote!(#unescaped: #value)
            });

            quote! {
                #unpacked {
                    #(#fields ,)*
                }
            }
        }
        ir::Data::Enum(r#enum) => {
            let discriminant = r#enum.discriminant();

            let variants = r#enum.variants.iter().map(|variant| {
                assert!(!variant.extract, "TODO");

                let fields = variant.r#struct.fields.iter().map(|field| {
                    let name = &field.ident;
                    let value = field.ty.unpack(crate::gen::get::get_field(
                        ty_struct,
                        field,
                        (discriminant.size + field.offset) as u8,
                    ));

                    quote!(#name: #value)
                });

                let discriminant = ty_struct
                    .as_tight()
                    .loosen()
                    .literal(variant.discriminant as u128);

                let ident = &variant.r#struct.unpacked;

                quote!(#discriminant => #unpacked::#ident { #(#fields ,)* })
            });

            let discriminant = lift::Expr::new(quote!(self.value), ty_struct)
                .discriminant(&discriminant)
                .compile();

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

    quote! {
        impl #generics_impl ::ribbit::Unpack for #packed #generics_ty #generics_where {
            type Unpacked = #unpacked #generics_ty;

            #[inline]
            fn unpack(self) -> #unpacked #generics_ty {
                #unpack
            }
        }
    }
}
