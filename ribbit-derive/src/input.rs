use darling::ast::Fields;
use darling::util::SpannedValue;
use darling::FromDeriveInput;
use darling::FromField;
use darling::FromVariant;

use crate::ir;

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(ribbit), forward_attrs(doc, derive))]
pub struct Item {
    #[darling(flatten)]
    pub(crate) opt: ir::StructOpt,

    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: darling::ast::Data<Variant, SpannedValue<Field>>,
}

#[derive(FromVariant, Clone, Debug)]
#[darling(attributes(ribbit), forward_attrs(doc, derive))]
pub(crate) struct Variant {
    #[darling(flatten)]
    pub(crate) opt: ir::StructOpt,

    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) ident: syn::Ident,
    pub(crate) fields: Fields<SpannedValue<Field>>,
}

#[derive(FromField, Clone, Debug)]
#[darling(attributes(ribbit))]
pub(crate) struct Field {
    #[darling(flatten)]
    pub(crate) opt: ir::FieldOpt,

    pub(crate) vis: syn::Visibility,
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) ty: syn::Type,
}
