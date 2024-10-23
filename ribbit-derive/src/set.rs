use proc_macro2::Literal;
use quote::format_ident;
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
        let source = self.field.repr();
        let target = self.repr;

        let ident = self.field.ident().unwrap();

        // Convert from field type to field native type
        let field_native = source.convert_to_native(ident);

        // Widen from field native type to struct native type
        let struct_native = match (source.as_native(), target.as_native()) {
            (field, r#struct) if field == r#struct => field_native,
            // FIXME: handle non-native struct type
            (_, r#struct) => quote!((#field_native as #r#struct)),
        };

        // Left shift
        let offset = self.field.offset();
        let shifted = match offset {
            0 => struct_native,
            _ => quote!((#struct_native << #offset)),
        };

        // Clear existing data in field
        let field_mask = !(source.mask() << offset);
        let struct_mask = target.mask();
        let clear = match field_mask & struct_mask {
            0 => None,
            mask => Some(Literal::usize_unsuffixed(mask)),
        };

        let value = target.convert_to_native(quote!(self.value));
        let r#struct = target.convert_from_native(match clear {
            None => quote!((#shifted)),
            Some(clear) => quote!((#value & #clear | #shifted)),
        });

        let vis = self.field.vis();
        let with_ident = format_ident!("with_{}", ident);
        quote! {
            #vis const fn #with_ident(&self, #ident: #source) -> Self {
                Self {
                    value: #r#struct,
                }
            }
        }
        .to_tokens(tokens)
    }
}
