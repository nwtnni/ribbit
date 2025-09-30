use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn pack(ir: &ir::Ir) -> TokenStream {
    let unpacked = ir.ident_unpacked();
    let packed = ir.ident_packed();

    let pack = match &ir.data {
        ir::Data::Struct(r#struct) => {
            let arguments = r#struct
                .iter_nonzero()
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

                let suffix = variant.r#struct.unpacked.to_string().to_snake_case();
                let new = ir.opt().new.name(Some(&suffix));

                let arguments = variant.r#struct.fields.iter().map(|field| {
                    let name = field.ident.escape();
                    field.r#type.pack(quote!(#name))
                });

                let variant = &variant.r#struct.unpacked;
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

    let generics = ir.generics_bounded();
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
