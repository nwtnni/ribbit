use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::Or;
use crate::Type;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct FieldOpt {
    #[darling(flatten)]
    common: ir::CommonOpt,
    #[darling(default)]
    skip: bool,
}

impl FieldOpt {
    pub(crate) fn name<'ir>(field: &'ir ir::Field) -> Cow<'ir, syn::Ident> {
        field.opt.get.common.rename_with(|| field.ident.escaped())
    }
}

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ir::Data::Struct(r#struct) = &ir.data else {
        return Or::L(core::iter::empty());
    };

    let precondition = crate::gen::pre::precondition();

    Or::R({
        r#struct
            .iter()
            .filter(|field| !field.opt.get.skip)
            .map(move |field| {
                let value = get_field(
                    &r#struct.r#type,
                    field,
                    r#struct.max_offset,
                    field.offset as u8,
                );
                let vis = field.opt.get.common.vis(field.vis);
                let name = FieldOpt::name(field);
                let r#type = field.r#type.packed();

                quote! {
                    #[inline]
                    #vis const fn #name(self) -> #r#type {
                        #precondition
                        #value
                    }
                }
            })
    })
}

pub(crate) fn get_field(
    r#type: &Type,
    field: &ir::Field,
    max_offset: usize,
    offset: u8,
) -> TokenStream {
    let expr = lift::Expr::value_self(r#type).shift_right(offset);

    // Loose type will be implicitly truncated by `as` cast
    match field.r#type.is_loose()
        // No other fields to mask
        || offset as usize == max_offset
    {
        true => expr,
        false => expr.and(field.r#type.mask()),
    }
    .compile(&*field.r#type)
}
