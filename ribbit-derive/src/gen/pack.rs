use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn pack(item: &ir::Item) -> TokenStream {
    let unpacked = item.ident_unpacked();
    let packed = item.ident_packed();

    let pack = match &item.data {
        ir::Data::Struct(r#struct) => {
            let arguments = r#struct
                .iter()
                .filter(|field| !field.r#type.is_zst())
                .map(|ir::Field { ident, r#type, .. }| r#type.pack(quote!(self.#ident)));

            quote!(#packed::new(#(#arguments),*))
        }
        ir::Data::Enum(r#enum) => {
            let variants = r#enum.variants.iter().map(|variant| {
                let patterns = variant
                    .r#struct
                    .fields
                    .iter()
                    .map(|field| field.ident.pattern());

                let new = item.opt().new.name(Some(variant.ident));

                let arguments = variant.r#struct.fields.iter().map(|field| {
                    let name = field.ident.escape();
                    field.r#type.pack(quote!(#name))
                });

                let variant = &variant.ident;
                quote! {
                    Self::#variant { #(#patterns ,)* } => #packed::#new( #(#arguments ,)* )
                }
            });

            quote! {
                match self {
                    #(#variants ,)*
                }
            }
        }
    };

    let generics = item.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    quote! {
        unsafe impl #generics_impl ::ribbit::Pack for #unpacked #generics_type #generics_where {
            type Packed = #packed #generics_type;
            #[inline]
            fn pack(self) -> #packed #generics_type {
                #pack
            }
        }
    }
}
