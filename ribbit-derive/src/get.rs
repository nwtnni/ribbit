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
        let fields = self.0.fields().iter().map(Field);
        quote! {
            impl #ident {
                #( #fields )*
            }
        }
        .to_tokens(tokens)
    }
}

struct Field<'ir>(&'ir ir::Field<'ir>);

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = self.0.vis();
        let ty = self.0.ty();
        let ident = self.0.ident();
        let offset = self.0.offset();

        let body = match ty {
            ir::Type::Builtin { .. } => {
                let shifted = match offset {
                    0 => quote! { self.value },
                    _ => quote! { (self.value >> #offset) },
                };

                match ty {
                    ir::Type::Builtin { ident, builtin: _ } => quote! {
                        #shifted as #ident
                    },
                }
            }
        };

        quote! {
            #vis const fn #ident(&self) -> #ty {
                #body
            }
        }
        .to_tokens(tokens)
    }
}
