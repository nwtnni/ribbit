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
use ir::Ir;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
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

    let mut stream = TokenStream::new();
    let parent = input::Item::from_derive_input(&input)?;
    let parent_ir = ir::new(&parent)?;

    stream.append_all(pack_item(&parent_ir)?);

    match &parent.data {
        darling::ast::Data::Struct(_) => (),
        darling::ast::Data::Enum(r#enum) => {
            for variant in r#enum {
                match variant.fields.as_shape() {
                    // Assume wrapped type implements `ribbit::Pack`
                    Shape::Newtype => (),

                    // No generation needed
                    Shape::Unit => (),

                    // Generate struct for variant
                    Shape::Named | Shape::Tuple => {
                        let child = input::Item {
                            opt: variant.opt.clone(),
                            attrs: variant.attrs.clone(),
                            vis: parent.vis.clone(),
                            ident: variant.ident.clone(),
                            generics: parent.generics.clone(),
                            data: darling::ast::Data::Struct(variant.fields.clone()),
                        };
                        let child_ir = ir::new(&child)?;

                        stream.append_all(pack_item(&child_ir)?);

                        // Use all of parent's where clauses here.
                        let generics = parent_ir.generics_bounded(None);
                        let (r#impl, ty, r#where) = generics.split_for_impl();
                        let from = &variant.ident;
                        let into = &parent.ident;
                        // FIXME: can't access methods in parent IR here
                        let unpacked = format_ident!("{}Unpacked", into);
                        let new = parent.opt.new.name();

                        // Generate `From` implementations to unpacked and packed types
                        stream.append_all(quote!(
                            impl #r#impl From<#from #ty> for #unpacked #ty #r#where {
                                fn from(variant: #from #ty) -> Self {
                                    #unpacked::#from(variant)
                                }
                            }

                            impl #r#impl From<#from #ty> for #into #ty #r#where {
                                fn from(variant: #from #ty) -> Self {
                                    #into::#new(#unpacked::#from(variant))
                                }
                            }
                        ));
                    }
                }
            }
        }
    }

    Ok(stream)
}

fn pack_item(ir: &Ir) -> Result<TokenStream, darling::Error> {
    let pre = gen::pre(ir);
    let repr = gen::repr(ir);
    let new = gen::new(ir);
    let get = gen::get(ir);
    let set = gen::set(ir);
    let copy = gen::copy(ir);
    let debug = gen::debug(ir);

    let generics = ir.generics_bounded(None);
    let (r#impl, ty, r#where) = generics.split_for_impl();
    let ident = &ir.ident;

    Ok(quote! {
        #repr

        impl #r#impl #ident #ty #r#where {
            #new

            #(#get)*

            #(#set)*

            #pre
        }

        #copy

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
