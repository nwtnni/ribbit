use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::leaf;
use crate::Leaf;
use crate::Spanned;

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
    repr: &'ir Spanned<Leaf>,
    field: &'ir ir::Field<'ir>,
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let source = self.repr;
        let target = self.field.repr();

        // Convert from struct type to struct native type
        let native = source.convert_to_native(quote!(self.value));

        // Right shift
        let shifted = match self.field.offset() {
            0 => native,
            offset => quote!((#native >> #offset)),
        };

        // Narrow from struct native type to field native type
        let narrowed = match (source.as_native(), target.as_native()) {
            (r#struct, field) if field == r#struct => shifted,
            (_, field) => quote!(#shifted as #field),
        };

        // Convert from field native type to field type
        let field = target.convert_from_native(narrowed);

        let vis = self.field.vis();
        let ident = self.field.ident();
        quote! {
            #vis const fn #ident(&self) -> #target {
                #field
            }
        }
        .to_tokens(tokens)
    }
}
