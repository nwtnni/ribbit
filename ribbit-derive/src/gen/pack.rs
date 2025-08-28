use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

pub(crate) fn pack(ir: &ir::Ir) -> TokenStream {
    let unpacked = ir.ident_unpacked();
    let packed = ir.ident_packed();

    let pack = match &ir.data {
        ir::Data::Struct(r#struct) => {
            let arguments = r#struct
                .iter_nonzero()
                .map(|ir::Field { ident, ty, .. }| ty.pack(quote!(self.#ident)));

            quote!(#packed::new(#(#arguments),*))
        }
        ir::Data::Enum(r#enum) => {
            let variants = r#enum.variants.iter().map(|variant| {
                assert!(!variant.extract, "TODO");

                let patterns = variant.r#struct.fields.iter().map(|field| {
                    let name = field.ident.unescaped("");
                    let value = field.ident.escaped();
                    quote!(#name: #value)
                });

                let new = format_ident!(
                    "{}_{}",
                    ir.opt().new.name(),
                    variant.r#struct.unpacked.to_string().to_snake_case(),
                );

                let arguments = variant.r#struct.fields.iter().map(|field| {
                    let name = field.ident.escaped();
                    field.ty.pack(quote!(#name))
                });

                let ident = &variant.r#struct.unpacked;
                // FIXME: support shorthand
                quote! {
                    #[allow(non_shorthand_field_patterns)]
                    Self::#ident { #(#patterns ,)* } => #packed::#new( #(#arguments ,)* )
                }
            });

            quote! {
                match self {
                    #(#variants ,)*
                }
            }
        }
    };

    let tight = ir.tight();
    let size = tight.size();

    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    quote! {
        unsafe impl #generics_impl ::ribbit::Pack for #unpacked #generics_ty #generics_where {
            const BITS: usize = #size;
            type Packed = #packed #generics_ty;
            type Loose = <#tight as ::ribbit::Pack>::Loose;

            #[inline]
            fn pack(self) -> #packed #generics_ty {
                #pack
            }
        }
    }
}
