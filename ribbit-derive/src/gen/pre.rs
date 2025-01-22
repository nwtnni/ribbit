use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;

use crate::ir;

pub(crate) fn pre(ir::Ir { data, tight, .. }: &ir::Ir) -> TokenStream {
    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            let fields = fields
                .iter()
                .map(|field| &field.ty)
                .filter(|ty| ty.is_node());

            let nonzero = fields
                .clone()
                .filter(|ty| ty.nonzero())
                .map(|ty| quote!(::ribbit::private::assert_impl_all!(<#ty as ::ribbit::Pack>::Tight: ::ribbit::NonZero)));

            let pack = fields.map(|ty| {
                let size = ty.size();
                let expected = match *size {
                    0 => quote!(::core::mem::size_of::<#ty>()),
                    _ => quote!(<#ty as ::ribbit::Pack>::BITS),
                };
                quote_spanned! {size.span()=>
                    if #size > #expected {
                        panic!(concat!(
                            "Annotated size ",
                            stringify!(#size),
                            " does not match actual size of type ",
                            stringify!(#ty),
                        ));
                    }
                }
            });

            quote! {
                #[doc(hidden)]
                const _RIBBIT_ASSERT_LAYOUT: () = {
                    #(#nonzero;)*
                    #(#pack)*
                };
            }
        }
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let variants = variants
                .iter()
                .filter_map(|variant| variant.ty.as_ref())
                .filter(|ty| ty.is_node());

            let nonzero = variants
                .clone()
                .filter(|ty| ty.nonzero())
                .map(|ty| quote!(::ribbit::private::assert_impl_all!(#ty: ::ribbit::NonZero)));

            let size_enum = tight.size();
            let size_discriminant = r#enum.discriminant_size();
            let size_variant = *size_enum - size_discriminant;

            let pack = variants.map(|ty| {
                let size = ty.size();
                quote_spanned! {size.span()=>
                    if #size != <#ty as ::ribbit::Pack>::BITS {
                        panic!(concat!(
                            "Annotated size ",
                            stringify!(#size),
                            " does not match actual size of type ",
                            stringify!(#ty),
                        ));
                    }

                    if #size > #size_variant {
                        panic!(concat!(
                            "Type ",
                            stringify!(#ty),
                            " of size ",
                            stringify!(#size),
                            " does not fit in enum of size ",
                            stringify!(#size_enum),
                            " with discriminant size ",
                            stringify!(#size_discriminant),
                        ));
                    }
                }
            });

            quote! {
                #[doc(hidden)]
                const _RIBBIT_ASSERT_LAYOUT: () = {
                    #(#nonzero;)*
                    #(#pack)*
                };
            }
        }
    }
}
