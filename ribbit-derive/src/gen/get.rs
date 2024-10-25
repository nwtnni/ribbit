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
        let source = **self.repr;
        let target = *self.field.repr;

        let field = lift::lift(quote!(self.value), source)
            .into_native()
            .apply(lift::Op::Shift {
                dir: lift::Dir::R,
                shift: self.field.offset(),
            })
            .into_repr(target);

        let vis = self.field.vis;
        let ident = self.field.ident.escaped();

        quote! {
            #vis const fn #ident(&self) -> #target {
                #field
            }
        }
        .to_tokens(tokens)
    }
}
