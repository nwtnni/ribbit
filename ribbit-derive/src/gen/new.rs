use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::lift::lift;
use crate::lift::LoosenExt as _;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt {
    rename: Option<syn::Ident>,
    vis: Option<syn::Visibility>,
}

pub(crate) fn new(
    ir::Ir {
        ident,
        opt,
        repr,
        vis,
        data,
        ..
    }: &ir::Ir,
) -> TokenStream {
    let new = opt
        .new
        .rename
        .clone()
        .unwrap_or_else(|| format_ident!("new"));
    let vis = opt.new.vis.as_ref().unwrap_or(vis);

    match data {
        ir::Data::Struct(ir::Struct { fields }) => {
            let parameters = fields.iter().map(|field| {
                let ident = field.ident.escaped();
                let ty = &field.ty;
                quote!(#ident: #ty)
            });

            let value = fields
                .iter()
                .fold(
                    Box::new(lift::constant(0, repr.loosen())) as Box<dyn lift::Loosen>,
                    |state, field| {
                        let ident = field.ident.escaped();
                        let value = lift::lift(ident, (*field.ty).clone())
                            .apply(lift::Op::Cast(repr.loosen()))
                            .apply(lift::Op::Shift {
                                dir: lift::Dir::L,
                                shift: field.offset,
                            });

                        Box::new(state.apply(lift::Op::Or(Box::new(value))))
                    },
                )
                .tighten(**repr);

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
        ir::Data::Enum(r#enum @ ir::Enum { variants }) => {
            let unpacked = r#enum.unpacked(ident);

            let discriminant_size = r#enum.discriminant_size();
            let loose = repr.loosen();

            let discriminants = variants.iter().enumerate().map(|(index, variant)| {
                let packed = lift::constant(index, loose).apply(lift::Op::Or(match &variant.ty {
                    None => Box::new(lift::constant(0, loose)) as Box<dyn lift::Loosen>,
                    Some(ty) => Box::new(
                        lift(quote!(inner), (**ty).clone())
                            .apply(lift::Op::Shift {
                                dir: lift::Dir::L,
                                shift: discriminant_size,
                            })
                            .apply(lift::Op::Cast(loose)),
                    ),
                }));

                let ident = &variant.ident;
                match &variant.ty {
                    None => quote!(#unpacked::#ident => #packed),
                    Some(_) => quote!(#unpacked::#ident(inner) => #packed),
                }
            });

            let value =
                lift::lift(quote!(match unpacked { #(#discriminants),* }), loose).tighten(**repr);

            quote! {
                #[inline]
                #vis const fn #new(
                    unpacked: #unpacked,
                ) -> Self {
                    let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                    Self {
                        value: #value,
                        r#type: ::ribbit::private::PhantomData,
                    }
                }
            }
        }
    }
}
