use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;

#[derive(FromMeta, Debug, Default)]
pub(crate) struct StructOpt {
    rename: Option<syn::Ident>,
    vis: Option<syn::Visibility>,
}

pub(crate) fn new(r#struct: &ir::Struct) -> TokenStream {
    let parameters = r#struct.fields.iter().map(|field| {
        let ident = field.ident.escaped();
        let ty = &field.ty;
        quote!(#ident: #ty)
    });

    let value_struct = r#struct
        .fields
        .iter()
        .fold(
            Box::new(lift::zero(r#struct.repr.to_native())) as Box<dyn lift::Native>,
            |state, field| {
                let ident = field.ident.escaped();
                let value_field = lift::lift(ident, (*field.ty).clone())
                    .convert_to_native()
                    .apply(lift::Op::Cast(r#struct.repr.to_native()))
                    .apply(lift::Op::Shift {
                        dir: lift::Dir::L,
                        shift: field.offset,
                    });

                Box::new(state.apply(lift::Op::Or(Box::new(value_field))))
            },
        )
        .convert_to_ty(*r#struct.repr);

    let ident = r#struct.ident;
    let opt = &r#struct.opt.new;
    let new = opt.rename.clone().unwrap_or_else(|| format_ident!("new"));
    let vis = opt.vis.as_ref().unwrap_or(r#struct.vis);

    quote! {
        impl #ident {
            #vis const fn #new(
                #(#parameters),*
            ) -> Self {
                Self { value: #value_struct }
            }
        }
    }
}
