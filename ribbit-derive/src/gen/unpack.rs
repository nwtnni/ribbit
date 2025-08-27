use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift::Lift as _;

pub(crate) fn unpack(ir: &ir::Ir) -> TokenStream {
    let ty_struct = ir.tight();

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
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => {
            todo!()
            //     let variants = variants.iter().enumerate().map(|(index, variant)| {
            //         let discriminant = tight.loosen().literal(index as u128);
            //         let ident = &variant.ident;
            //         let value = match variant.ty.as_deref().cloned() {
            //             None => quote!(#unpacked::#ident),
            //             Some(ty_variant) => {
            //                 #[allow(clippy::precedence)]
            //                 let inner = (quote!(self.value).lift() % ty_struct.clone()
            //                     >> r#enum.discriminant_size())
            //                     % ty_variant;
            //
            //                 quote!(#unpacked::#ident(#inner))
            //             }
            //         };
            //
            //         quote!(#discriminant => #value)
            //     });
            //
            //     let discriminant =
            //         (quote!(self.value).lift() % ty_struct.clone()) & r#enum.discriminant_mask();
            //
            //     quote! {
            //         match #discriminant {
            //             #(#variants,)*
            //             _ => unsafe {
            //                 ::core::hint::unreachable_unchecked()
            //             }
            //         }
            //     }
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
