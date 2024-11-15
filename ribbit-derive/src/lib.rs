mod error;
mod gen;
mod input;
mod ir;
mod lift;
mod ty;

use core::ops::Deref;
use core::ops::DerefMut;

use darling::util::AsShape;
use darling::util::Shape;
use darling::util::SpannedValue;

pub(crate) use error::Error;

use darling::FromDeriveInput as _;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use quote::TokenStreamExt as _;
use syn::parse_macro_input;
use syn::parse_quote;

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

fn pack_inner(
    attr: TokenStream,
    mut input: syn::DeriveInput,
) -> Result<TokenStream, darling::Error> {
    input.attrs.push(parse_quote!(#[ribbit(#attr)]));

    let mut item = input::Item::from_derive_input(&input)?;
    let mut stream = TokenStream::new();

    match &item.data {
        darling::ast::Data::Enum(r#enum) => {
            for variant in r#enum {
                match variant.fields.as_shape() {
                    // Generate
                    Shape::Named | Shape::Tuple => {
                        let mut item = input::Item {
                            opt: variant.opt.clone(),
                            attrs: variant.attrs.clone(),
                            vis: item.vis.clone(),
                            ident: variant.ident.clone(),
                            generics: syn::Generics::default(),
                            data: darling::ast::Data::Struct(variant.fields.clone()),
                        };

                        stream.append_all(pack_item(&mut item)?);
                    }

                    Shape::Newtype | Shape::Unit => (),
                }
            }

            stream.append_all(pack_item(&mut item)?);
        }
        darling::ast::Data::Struct(_) => {
            stream.append_all(pack_item(&mut item)?);
        }
    }

    Ok(stream)
}

fn pack_item(item: &mut input::Item) -> Result<TokenStream, darling::Error> {
    let ir = ir::new(item)?;

    let pre = gen::pre(&ir);
    let repr = gen::repr(&ir);
    let new = gen::new(&ir);
    let get = gen::get(&ir);
    let set = gen::set(&ir);
    let debug = gen::debug(&ir);

    let (r#impl, ty, r#where) = ir.generics.split_for_impl();
    let ident = &ir.ident;

    Ok(quote! {
        #repr

        impl #r#impl #ident #ty #r#where {
            #new

            #(#get)*

            #(#set)*

            #pre
        }

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
