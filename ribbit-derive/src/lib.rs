mod error;
mod get;
mod input;
mod ir;
mod leaf;
mod set;

use core::ops::Deref;
use core::ops::DerefMut;

use darling::util::SpannedValue;

pub(crate) use error::Error;
pub(crate) use leaf::Leaf;

use darling::FromDeriveInput as _;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn pack(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);

    pack_inner(attr.into(), item)
        .unwrap_or_else(|error| error.write_errors())
        .into()
}

fn pack_inner(attr: TokenStream, input: syn::DeriveInput) -> Result<TokenStream, darling::Error> {
    let attr = input::Attr::new(attr)?;
    let item = input::Item::from_derive_input(&input)?;

    let ir = ir::new(&attr, &input, &item)?;
    Ok(Output { ir }.to_token_stream())
}

struct Output<'input> {
    ir: ir::Struct<'input>,
}

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ir = &self.ir;
        let get = crate::get::Struct::new(ir);
        let set = crate::set::Struct::new(ir);

        let output = quote! {
            #ir

            #get

            #set
        };

        output.to_tokens(tokens);
    }
}

pub(crate) struct Spanned<T>(SpannedValue<T>);

impl<T> Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ToTokens> ToTokens for Spanned<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        #[allow(clippy::explicit_auto_deref)]
        let inner: &T = &*self.0;
        quote_spanned!(self.0.span()=> #inner).to_tokens(tokens)
    }
}

impl<T> From<SpannedValue<T>> for Spanned<T> {
    fn from(inner: SpannedValue<T>) -> Self {
        Self(inner)
    }
}
