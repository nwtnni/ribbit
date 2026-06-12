use core::iter;
use std::borrow::Cow;

use darling::FromMeta;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;
use crate::lift;
use crate::r#type::Tight;
use crate::Or;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt(ir::CommonOpt);

pub(crate) fn from_raw_unchecked<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let opt = &ir.opt().from_raw_unchecked;
    let vis = opt.0.vis(&ir.vis);
    let tight = ir.r#type().as_tight();

    if opt.0.skip {
        Or::L(core::iter::empty())
    } else {
        Or::R(iter::once(from_raw_unchecked_struct(
            vis,
            &ir.opt().from_raw_unchecked.name(None),
            tight,
            |expr| expr.compile(tight),
        )))
    }
    .chain(match &ir.data {
        ir::Data::Struct(_) => Or::L(core::iter::empty()),
        ir::Data::Enum(r#enum @ ir::Enum { variants, .. }) => {
            Or::R(variants.iter().filter_map(move |variant| {
                let opt = &variant.r#struct.opt.from_raw_unchecked;
                if opt.0.skip {
                    return None;
                }

                let compile = |expr: lift::Expr| {
                    lift::Expr::or([
                        lift::Expr::constant(variant.discriminant as u128),
                        expr.shift_left(r#enum.discriminant.size as u8),
                    ])
                    .compile(tight)
                };

                Some(from_raw_unchecked_struct(
                    opt.0.vis(vis),
                    &opt.name(Some(variant.r#struct.unpacked)),
                    variant.r#struct.r#type.as_tight(),
                    compile,
                ))
            }))
        }
    })
}

fn from_raw_unchecked_struct<'ir, F: FnOnce(lift::Expr<'ir>) -> TokenStream>(
    vis: &syn::Visibility,
    name: &syn::Ident,
    tight: &'ir Tight,
    compile: F,
) -> TokenStream {
    let precondition = crate::gen::precondition::assert();
    let raw = compile(lift::Expr::value_tight(quote!(raw), tight));

    quote! {
        #[inline]
        #vis const unsafe fn #name(raw: #tight) -> Self {
            #precondition
            Self {
                value: #raw,
                r#type: ::ribbit::PhantomData,
            }
        }
    }
}

impl StructOpt {
    fn name<'ir>(&'ir self, variant: Option<&syn::Ident>) -> Cow<'ir, syn::Ident> {
        self.0.rename_with(|| match variant {
            None => Cow::Owned(format_ident!("from_raw_unchecked")),
            Some(variant) => Cow::Owned(format_ident!(
                "{}_from_raw_unchecked",
                variant.to_string().to_snake_case(),
            )),
        })
    }
}
