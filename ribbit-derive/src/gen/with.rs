use core::iter;
use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt(ir::CommonOpt);

impl FieldOpt {
    fn name<'ir>(field: &'ir ir::Field) -> Cow<'ir, syn::Ident> {
        field
            .opt
            .with
            .0
            .rename_with(|| Cow::Owned(field.ident.prefix("with")))
    }
}

pub(crate) fn with<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ir::Data::Struct(r#struct) = &ir.data else {
        return Or::L(iter::empty());
    };

    Or::R(
        r#struct
            .iter_nonzero()
            .filter(|field| !field.opt.with.0.skip)
            .map(move |field| {
                let value = lift::Expr::or([
                    lift::Expr::value(field.ident.escape(), &field.r#type)
                        .shift_left(field.offset as u8),
                    lift::Expr::value_self(&r#struct.r#type).and(
                        !(field.r#type.mask() << field.offset) & r#struct.r#type.as_tight().mask(),
                    ),
                ])
                .compile(ir.r#type().as_tight());

                let vis = field.opt.with.0.vis(field.vis);
                let with = FieldOpt::name(field);
                let name = field.ident.escape();
                let r#type = field.r#type.packed();
                let precondition = crate::gen::precondition::assert();

                quote! {
                    #[inline]
                    #vis const fn #with(self, #name: #r#type) -> Self {
                        #precondition
                        Self {
                            value: #value,
                            r#type: ::ribbit::private::PhantomData,
                        }
                    }
                }
            }),
    )
}
