mod error;
mod gen;
mod input;
mod ir;
mod lift;
mod ty;

use core::ops::Deref;
use core::ops::DerefMut;

use darling::util::SpannedValue;

pub(crate) use error::Error;

use darling::FromDeriveInput as _;
use proc_macro2::Span;
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

    let pre = gen::pre::Struct(&ir);
    let repr = gen::repr::Struct(&ir);
    let new = gen::new(&attr.new, &ir);
    let get = gen::get::Struct(&ir);
    let set = gen::set::Struct(&ir);
    let debug = gen::debug::Struct(&ir);

    Ok(quote! {
        #pre

        #repr

        #new

        #get

        #set

        #debug
    }
    .to_token_stream())
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Spanned<T>(SpannedValue<T>);

impl<T> Spanned<T> {
    pub(crate) fn new(inner: T, span: Span) -> Self {
        Self(SpannedValue::new(inner, span))
    }

    pub(crate) fn span(&self) -> Span {
        self.0.span()
    }

    pub(crate) fn map_ref<F: FnOnce(&T) -> U, U>(&self, apply: F) -> Spanned<U> {
        Spanned(self.0.map_ref(apply))
    }
}

impl<T> From<T> for Spanned<T> {
    fn from(inner: T) -> Self {
        Spanned(SpannedValue::new(inner, Span::call_site()))
    }
}

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
