use quote::format_ident;
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
        let repr = self.field.repr();
        let ident = self.field.ident().unwrap();

        // Convert from field type to field native type
        let field_native = match repr.ty() {
            ir::Tree::Leaf(ir::Leaf::Native(_)) => quote!(#ident),
            ir::Tree::Leaf(ir::Leaf::Arbitrary(_)) => quote!(#ident.value()),
        };

        // Widen from field native type to struct native type
        let struct_native = match (repr.ty().as_native(), self.repr.ty().as_native()) {
            (field, r#struct) if field == r#struct => field_native,
            (_, r#struct) => quote!((#field_native as #r#struct)),
        };

        // Left shift
        let offset = self.field.offset();
        let shift = match offset {
            0 => struct_native,
            _ => quote!((#struct_native << #offset)),
        };

        // Clear existing data in field
        let field_mask = !(ir::mask(repr.ty().size()) << offset);
        let struct_mask = ir::mask(self.repr.ty().size());
        let clear = match field_mask & struct_mask {
            0 => None,
            mask => Some(ir::literal(self.repr.ty().as_native(), mask)),
        };

        let r#struct = match (clear, self.repr.ty()) {
            (None, ir::Leaf::Native(_)) => shift,
            (None, ir::Leaf::Arbitrary(arbitrary)) => quote!(#arbitrary::new(#shift)),
            (Some(clear), ir::Leaf::Native(_)) => quote!(self.value & #clear | #shift),
            (Some(clear), ir::Leaf::Arbitrary(arbitrary)) => {
                quote!(#arbitrary::new(self.value.value() & #clear | #shift))
            }
        };

        let vis = self.field.vis();
        let with_ident = format_ident!("with_{}", ident);
        quote! {
            #vis const fn #with_ident(&self, #ident: #repr) -> Self {
                Self {
                    value: #r#struct,
                }
            }
        }
        .to_tokens(tokens)
    }
}
