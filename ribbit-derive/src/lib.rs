use darling::FromDeriveInput;
use darling::FromField;
use darling::FromMeta;
use darling::FromVariant;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn pack(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);

    pack_inner(attr.into(), item)
        .map(|output| output.to_token_stream())
        .unwrap_or_else(|error| error.write_errors())
        .into()
}

fn pack_inner(attr: TokenStream, item: syn::DeriveInput) -> Result<Output, darling::Error> {
    let input = darling::ast::NestedMeta::parse_meta_list(attr)
        .map_err(darling::Error::from)
        .and_then(|meta| Input::from_list(&meta))?;

    let item = Item::from_derive_input(&item)?;

    Ok(Output {})
}

#[derive(FromMeta)]
struct Input {
    size: usize,
}

#[derive(FromDeriveInput, Debug)]
struct Item {
    ident: syn::Ident,
    data: darling::ast::Data<Variant, Field>,
}

struct Output {}

impl ToTokens for Output {
    fn to_tokens(&self, _tokens: &mut TokenStream) {}
}

#[derive(FromVariant, Debug)]
struct Variant {}

#[derive(FromField, Debug)]
struct Field {}
