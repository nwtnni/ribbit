use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift;
use crate::lift::NativeExt as _;
use crate::ty::Leaf;
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
        let ty_struct = **self.repr;
        let ty_field = &*self.field.ty;

        let value_field = lift::lift(quote!(self.value), ty_struct)
            .convert_to_native()
            .apply(lift::Op::Shift {
                dir: lift::Dir::R,
                shift: self.field.offset,
            })
            .convert_to_ty(ty_field.clone());

        let vis = self.field.vis;
        let ident = self.field.ident.escaped();

        quote! {
            #vis const fn #ident(&self) -> #ty_field {
                #value_field
            }
        }
        .to_tokens(tokens)
    }
}
