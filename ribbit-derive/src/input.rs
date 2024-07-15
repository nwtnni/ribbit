use darling::util::SpannedValue;
use darling::FromDeriveInput;
use darling::FromField;
use darling::FromMeta;
use darling::FromVariant;
use proc_macro2::TokenStream;

#[derive(FromMeta)]
pub(crate) struct Attr {
    pub(crate) size: SpannedValue<usize>,
}

impl Attr {
    pub(crate) fn new(attr: TokenStream) -> darling::Result<Self> {
        darling::ast::NestedMeta::parse_meta_list(attr)
            .map_err(darling::Error::from)
            .and_then(|meta| Self::from_list(&meta))
    }
}

#[derive(FromDeriveInput, Debug)]
pub struct Item {
    pub(crate) data: darling::ast::Data<Variant, SpannedValue<Field>>,
}

#[derive(FromVariant, Debug)]
pub(crate) struct Variant {}

#[derive(FromField, Debug)]
pub(crate) struct Field {
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) vis: syn::Visibility,
    pub(crate) ty: syn::Type,
}
