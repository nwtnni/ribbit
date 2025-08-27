use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::Or;

pub(crate) fn unpacked<'ir>(
    ir @ ir::Ir { vis, data, .. }: &'ir ir::Ir,
) -> impl Iterator<Item = TokenStream> + 'ir {
    let unpacked_ident = ir.ident_unpacked();
    let attrs = ir.attrs();

    let generics = ir.generics();
    let (_, generics_ty, generics_where) = generics.split_for_impl();

    // Generate unpacked type
    match data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.fields.iter().map(
                |ir::Field {
                     attrs,
                     ident,
                     ty,
                     vis,
                     ..
                 }| {
                    let ident = ident.is_named().then(|| ident.unescaped("")).into_iter();

                    quote! {
                        #(#attrs)*
                        #vis #( #ident: )* #ty
                    }
                },
            );

            let fields = if r#struct.fields.is_empty() {
                quote! { #generics_where ; }
            } else if r#struct.is_named() {
                quote! { #generics_where { #(#fields ,)* } }
            } else {
                quote! { ( #(#fields ,)* ) #generics_where; }
            };

            core::iter::once(quote! {
                #(#attrs)*
                #vis struct #unpacked_ident #generics_ty #fields
            })
        }
        ir::Data::Enum(r#enum) => {
            todo!()

            // let variants = r#enum.variants.iter().map(|variant| {
            //     if variant.extract
            //
            // })
        }
    }
}
