mod error;
mod gen;
mod input;
mod ir;
mod lift;
mod r#type;

use core::ops::Deref;
use core::ops::DerefMut;

use darling::util::SpannedValue;

pub(crate) use error::Error;
pub(crate) use r#type::Type;

use darling::FromDeriveInput as _;
use ir::Ir;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use quote::TokenStreamExt as _;
use syn::parse_macro_input;

#[proc_macro_derive(Pack, attributes(ribbit))]
pub fn pack(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    let mut output = TokenStream::new();
    match pack_impl(input, &mut output) {
        Ok(()) => output,
        Err(error) => error.write_errors(),
    }
    .into()
}

// Outer function (1) converts between proc_macro::TokenStream and proc_macro2::TokenStream and
// (2) handles errors by writing them out.
fn pack_impl(input: syn::DeriveInput, output: &mut TokenStream) -> Result<(), darling::Error> {
    let input = input::Item::from_derive_input(&input)?;
    let ir = Ir::new(&input)?;

    let pre = gen::pre(&ir);
    let new = gen::new(&ir);
    let nonzero = gen::nonzero(&ir);
    let pack = gen::pack(&ir);
    let packed = gen::packed(&ir);
    let unpack = gen::unpack(&ir);
    let get = gen::get(&ir);
    let set = gen::set(&ir);
    let from = gen::from(&ir);
    let debug = gen::debug(&ir);
    let hash = gen::hash(&ir);
    let eq = gen::eq(&ir);
    let ord = gen::ord(&ir);

    let generics = ir.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();
    let packed_ident = ir.ident_packed();

    output.append_all(quote! {
        #(#nonzero)*

        #pack

        #packed

        #unpack

        impl #generics_impl #packed_ident #generics_type #generics_where {
            #pre

            #(#new)*

            #(#get)*

            #(#set)*
        }

        #from
        #debug

        #hash
        #eq
        #ord
    });

    Ok(())
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

#[derive(Debug)]
pub(crate) enum Or<L, R> {
    L(L),
    R(R),
}

impl<L, R, T> Iterator for Or<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Or::L(l) => l.next(),
            Or::R(r) => r.next(),
        }
    }
}

fn mask(size: usize) -> u128 {
    assert!(
        size <= 128,
        "[INTERNAL ERROR]: cannot mask size > 128: {size}",
    );

    1u128
        .checked_shl(size as u32)
        .and_then(|mask| mask.checked_sub(1))
        .unwrap_or(u128::MAX)
}
