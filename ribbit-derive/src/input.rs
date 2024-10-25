use darling::util::SpannedValue;
use darling::FromDeriveInput;
use darling::FromField;
use darling::FromMeta;
use darling::FromVariant;
use proc_macro2::TokenStream;

use crate::ir;

#[derive(FromMeta, Debug)]
pub(crate) struct Attr {
    pub(crate) size: SpannedValue<usize>,
    pub(crate) nonzero: Option<SpannedValue<bool>>,
    #[darling(flatten)]
    pub(crate) opt: ir::StructOpt,
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
#[darling(attributes(ribbit))]
pub(crate) struct Field {
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) ty: syn::Type,
    pub(crate) nonzero: Option<SpannedValue<bool>>,
    pub(crate) size: Option<SpannedValue<usize>>,
    pub(crate) offset: Option<SpannedValue<usize>>,
    #[darling(flatten)]
    pub(crate) opt: ir::FieldOpt,
}
