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
        // Convert from struct type to struct native type
        let struct_native = match self.repr.ty() {
            ir::Leaf::Native(_) => quote!(self.value),
            ir::Leaf::Arbitrary(_) => {
                quote!(self.value.value())
            }
        };

        // Right shift
        let offset = self.field.offset();
        let shift = match offset {
            0 => struct_native,
            _ => quote!((#struct_native >> #offset)),
        };

        // Narrow from struct native type to field native type
        let repr = self.field.repr();
        let field_native = match (self.repr.ty().as_native(), repr.ty().as_native()) {
            (r#struct, field) if field == r#struct => shift,
            (_, field) => quote!(#shift as #field),
        };

        // Convert from field native type to field type
        let field = match repr.ty() {
            ir::Tree::Leaf(ir::Leaf::Native(_)) => field_native,
            ir::Tree::Leaf(ir::Leaf::Arbitrary(arbitrary)) => {
                let size = arbitrary.size();
                let mask = ir::literal(arbitrary.as_native(), ir::mask(size));
                quote!(#arbitrary::new(#field_native & #mask))
            }
        };

        let vis = self.field.vis();
        let ident = self.field.ident();
        quote! {
            #vis const fn #ident(&self) -> #repr {
                #field
            }
        }
        .to_tokens(tokens)
    }
}
