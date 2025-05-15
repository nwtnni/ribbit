use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug)]
pub(crate) struct StructOpt;

pub(crate) fn from(ir @ ir::Ir { item, parent, .. }: &ir::Ir) -> TokenStream {
    let Some(StructOpt) = &item.opt.from else {
        return TokenStream::new();
    };

    let generics = match parent {
        None => ir.generics_bounded(None),
        Some(parent) => parent.generics_bounded(None),
    };

    let (r#impl, ty, r#where) = generics.split_for_impl();

    match parent {
        None => {
            let packed = &item.ident;
            let unpacked = ir::Enum::unpacked(packed);
            let new = item.opt.new.name();

            quote! {
                impl #r#impl From<#unpacked #ty> for #packed #ty #r#where {
                    #[inline]
                    fn from(unpacked: #unpacked #ty) -> Self {
                        Self::#new(unpacked)
                    }
                }
            }
        }
        Some(parent) => {
            let variant = &item.ident;
            let packed = &parent.item.ident;
            let unpacked = ir::Enum::unpacked(packed);
            let new = parent.item.opt.new.name();

            quote!(
                impl #r#impl From<#variant #ty> for #unpacked #ty #r#where {
                    #[inline]
                    fn from(variant: #variant #ty) -> Self {
                        #unpacked::#variant(variant)
                    }
                }

                impl #r#impl From<#variant #ty> for #packed #ty #r#where {
                    #[inline]
                    fn from(variant: #variant #ty) -> Self {
                        #packed::#new(#unpacked::#variant(variant))
                    }
                }
            )
        }
    }
}
