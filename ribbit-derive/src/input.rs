use darling::ast::Fields;
use darling::util::SpannedValue;
use darling::FromDeriveInput;
use darling::FromField;
use darling::FromVariant;

use crate::ir;

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(ribbit), forward_attrs)]
pub struct Item {
    #[darling(flatten)]
    pub(crate) opt: ir::StructOpt,
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: darling::ast::Data<SpannedValue<Variant>, SpannedValue<Field>>,
}

#[derive(FromVariant, Clone, Debug)]
#[darling(attributes(ribbit), forward_attrs)]
pub(crate) struct Variant {
    #[darling(flatten)]
    pub(crate) opt: ir::StructOpt,
    pub(crate) attrs: Vec<syn::Attribute>,
    #[darling(default)]
    pub(crate) extract: bool,
    pub(crate) ident: syn::Ident,
    pub(crate) fields: Fields<SpannedValue<Field>>,
    pub(crate) discriminant: Option<syn::Expr>,
}

#[derive(FromField, Clone, Debug)]
#[darling(attributes(ribbit), forward_attrs)]
pub(crate) struct Field {
    #[darling(flatten)]
    pub(crate) opt: ir::FieldOpt,
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: Option<syn::Ident>,
    pub(crate) ty: syn::Type,
}
