use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;
use crate::repr::Leaf;
use crate::Spanned;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> Self {
        Self(r#struct)
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident;
        let fields = self.0.fields.iter().map(|field| Field {
            repr: &self.0.repr,
            field,
        });
        quote! {
            impl #ident {
                #( #fields )*
            }
        }
        .to_tokens(tokens)
    }
}

struct Field<'ir> {
    repr: &'ir Spanned<Leaf>,
    field: &'ir ir::Field<'ir>,
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let source = *self.field.repr;
        let target = **self.repr;

        let ident = &self.field.ident.escaped();
        let field = lift::lift(ident, source)
            .into_native()
            .apply(lift::Op::Cast(target.as_native()))
            .apply(lift::Op::Shift {
                dir: lift::Dir::L,
                shift: self.field.offset(),
            });

        let r#struct = lift::lift(quote!(self.value), target)
            .into_native()
            // Clear existing data
            .apply(lift::Op::And(
                !(source.mask() << self.field.offset()) & target.mask(),
            ))
            .apply(lift::Op::Or(Box::new(field)))
            .into_repr(target.into());

        let vis = self.field.vis;
        let with = self.field.ident.unescaped("with");
        quote! {
            #vis const fn #with(&self, #ident: #source) -> Self {
                Self {
                    value: #r#struct,
                }
            }
        }
        .to_tokens(tokens)
    }
}
