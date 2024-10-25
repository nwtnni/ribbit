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
            ty: &self.0.repr,
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
    ty: &'ir Spanned<Leaf>,
    field: &'ir ir::Field<'ir>,
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_field = &*self.field.ty;
        let ty_struct = **self.ty;

        let ident = &self.field.ident.escaped();
        let value_field = lift::lift(ident, ty_field.clone())
            .convert_to_native()
            .apply(lift::Op::Cast(ty_struct.to_native()))
            .apply(lift::Op::Shift {
                dir: lift::Dir::L,
                shift: self.field.offset,
            });

        let value_struct = lift::lift(quote!(self.value), ty_struct)
            .convert_to_native()
            // Clear existing data
            .apply(lift::Op::And(
                !(ty_field.mask() << self.field.offset) & ty_struct.mask(),
            ))
            .apply(lift::Op::Or(Box::new(value_field)))
            .convert_to_ty(ty_struct);

        let vis = self.field.vis;
        let with = self.field.ident.unescaped("with");
        quote! {
            #vis const fn #with(&self, #ident: #ty_field) -> Self {
                Self {
                    value: #value_struct,
                }
            }
        }
        .to_tokens(tokens)
    }
}
