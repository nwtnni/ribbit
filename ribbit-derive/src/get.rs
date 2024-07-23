use quote::quote;
use quote::ToTokens;

use crate::ir;

pub(crate) struct Struct<'ir>(&'ir ir::Struct<'ir>);

impl<'ir> Struct<'ir> {
    pub(crate) fn new(r#struct: &'ir ir::Struct<'ir>) -> Self {
        Self(r#struct)
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.0.ident();
        let fields = self.0.fields().iter().map(|field| Field {
            repr: self.0.repr(),
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
    repr: &'ir ir::StructRepr,
    field: &'ir ir::Field<'ir>,
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let value = match self.repr.ty() {
            ir::Leaf::Native(_) => quote!(self.value),
            ir::Leaf::Arbitrary(_) => {
                quote!(self.value.value())
            }
        };

        let offset = self.field.offset();
        let shift = match offset {
            0 => value,
            _ => quote!((#value >> #offset)),
        };

        let repr = self.field.repr();
        let cast = match (self.repr.ty().as_native(), repr.ty().as_native()) {
            (r#struct, field) if field == r#struct => shift,
            (_, field) => quote!(#shift as #field),
        };

        let body = match repr.ty() {
            ir::Tree::Leaf(ir::Leaf::Native(_)) => cast,
            ir::Tree::Leaf(ir::Leaf::Arbitrary(arbitrary)) => {
                let size = arbitrary.size();
                quote!(#arbitrary::new(#cast & ((1 << #size) - 1)))
            }
        };

        let vis = self.field.vis();
        let ident = self.field.ident();
        quote! {
            #vis const fn #ident(&self) -> #repr {
                #body
            }
        }
        .to_tokens(tokens)
    }
}
