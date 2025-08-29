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
            let size_discriminant = r#enum.discriminant_size();
            let variants = r#enum.variants.iter().enumerate().map(|(index, variant)| {
                let discriminant = ty_struct.as_tight().loosen().literal(index as u128);

                let ident = &variant.r#struct.unpacked;

                assert!(!variant.extract, "TODO");

                let fields = variant.r#struct.fields.iter().map(|field| {
                    let name = &field.ident;
                    let value = field.ty.unpack(crate::gen::get::get_field(
                        size_discriminant,
                        ty_struct,
                        field,
                    ));

                    quote!(#name: #value)
                });

                quote!(#discriminant => #unpacked::#ident { #(#fields ,)* })
            });

            let discriminant = lift::Expr::new(quote!(self.value), ty_struct)
                .mask(0, (1 << size_discriminant) - 1)
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
