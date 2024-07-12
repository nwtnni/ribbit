use darling::FromDeriveInput;
use darling::FromField;
use darling::FromMeta;
use darling::FromVariant;
use proc_macro2::TokenStream;

#[derive(FromMeta)]
pub(crate) struct Attr {
    pub(crate) size: u8,
}

impl Attr {
    pub(crate) fn from_token_stream(attr: TokenStream) -> darling::Result<Self> {
        darling::ast::NestedMeta::parse_meta_list(attr)
            .map_err(darling::Error::from)
            .and_then(|meta| Self::from_list(&meta))
    }
}

#[derive(FromDeriveInput, Debug)]
pub(crate) struct Item {
    pub(crate) ident: syn::Ident,
    pub(crate) data: darling::ast::Data<Variant, Field>,
}

#[derive(FromVariant, Debug)]
pub(crate) struct Variant {}

#[derive(FromField, Debug)]
pub(crate) struct Field {}
