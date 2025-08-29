use darling::util::Shape;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::Or;

pub(crate) fn unpacked<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let attrs = ir.attrs();
    let vis = &ir.vis;
    let ident = &ir.ident_unpacked();
    let generics = ir.generics();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L(core::iter::once(unpacked_struct(
            attrs, vis, ident, generics, r#struct,
        ))),
        ir::Data::Enum(r#enum) => {
            let variants = r#enum.variants.iter().map(|variant| {
                assert!(!variant.extract, "TODO");

                let attrs = &variant.r#struct.attrs;
                let ident = &variant.r#struct.unpacked;
                let fields = unpacked_fields(&variant.r#struct);

                quote! {
                    #(#attrs)*
                    #ident #fields
                }
            });

            let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

            let nonzero = r#enum
                .opt
                .nonzero
                .filter(|nonzero| **nonzero)
                .map(|_| quote!(unsafe impl #generics_impl ::ribbit::NonZero for #ident #generics_ty #generics_where {}))
                .into_iter();

            Or::R(core::iter::once(quote! {
                #(#attrs)*
                #vis enum #ident #generics_ty #generics_where {
                    #(#variants,)*
                }

                #(#nonzero)*
            }))
        }
    }
}

fn unpacked_struct(
    attrs: &[syn::Attribute],
    vis: &syn::Visibility,
    ident: &syn::Ident,
    generics: &syn::Generics,
    r#struct: &ir::Struct,
) -> TokenStream {
    let fields = unpacked_fields(r#struct);

    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    let fields = match r#struct.shape {
        Shape::Unit => quote! { #generics_where ; },
        Shape::Named => quote! { #generics_where #fields },
        Shape::Tuple | Shape::Newtype => quote! { #fields #generics_where; },
    };

    let nonzero = r#struct
        .opt
        .nonzero
        .filter(|nonzero| **nonzero)
        .map(|_| quote!(unsafe impl #generics_impl ::ribbit::NonZero for #ident #generics_ty #generics_where {}))
        .into_iter();

    quote! {
        #(#attrs)*
        #vis struct #ident #generics_ty #fields

        #(#nonzero)*
    }
}

fn unpacked_fields(r#struct: &ir::Struct) -> TokenStream {
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

    match r#struct.shape {
        Shape::Unit => TokenStream::new(),
        Shape::Tuple | Shape::Newtype => quote! {
            ( #(#fields ,)* )
        },
        Shape::Named => quote! {
            { #(#fields ,)* }
        },
    }
}
