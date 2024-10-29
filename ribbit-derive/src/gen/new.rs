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

pub(crate) fn new(
    ir::Struct {
        fields,
        opt,
        repr,
        vis,
        ..
    }: &ir::Struct,
) -> TokenStream {
    let parameters = fields.iter().map(|field| {
        let ident = field.ident.escaped();
        let ty = &field.ty;
        quote!(#ident: #ty)
    });

    let value = fields
        .iter()
        .fold(
            Box::new(lift::zero(repr.to_native())) as Box<dyn lift::Native>,
            |state, field| {
                let ident = field.ident.escaped();
                let value = lift::lift(ident, (*field.ty).clone())
                    .ty_to_native()
                    .apply(lift::Op::Cast(repr.to_native()))
                    .apply(lift::Op::Shift {
                        dir: lift::Dir::L,
                        shift: field.offset,
                    });

                Box::new(state.apply(lift::Op::Or(Box::new(value))))
            },
        )
        .native_to_ty(**repr);

    let new = opt
        .new
        .rename
        .clone()
        .unwrap_or_else(|| format_ident!("new"));
    let vis = opt.new.vis.as_ref().unwrap_or(vis);

    quote! {
        #[inline]
        #vis const fn #new(
            #(#parameters),*
        ) -> Self {
            let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
            Self {
                value: #value,
                r#type: ::ribbit::private::PhantomData,
            }
        }
    }
}
