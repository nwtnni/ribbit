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
    let input = parse_macro_input!(item as syn::DeriveInput);
    let mut output = TokenStream::new();
    match pack_impl(attr.into(), input, &mut output) {
        Ok(()) => output,
        Err(error) => error.write_errors(),
    }
    .into()
}

// Outer function (1) converts between proc_macro::TokenStream and proc_macro2::TokenStream and
// (2) handles errors by writing them out.
fn pack_impl(
    attr: TokenStream,
    mut input: syn::DeriveInput,
    output: &mut TokenStream,
) -> Result<(), darling::Error> {
    // Integrate with darling's attribute filter (namespace `ribbit` instead of `ribbit::pack`).
    input.attrs.push(parse_quote!(#[ribbit(#attr)]));

    let input = input::Item::from_derive_input(&input)?;
    let ir = ir::new(&input)?;
    pack_item(&ir, output)
}

// Generate code for a single item.
fn pack_item(ir: &Ir, output: &mut TokenStream) -> Result<(), darling::Error> {
    let pre = gen::pre(ir);
    let new = gen::new(ir);
    let unpacked = gen::unpacked(ir);
    let pack = gen::pack(ir);
    let packed = gen::packed(ir);
    let unpack = gen::unpack(ir);
    let get = gen::get(ir);
    let set = gen::set(ir);
    let from = gen::from(ir);
    let debug = gen::debug(ir);
    let hash = gen::hash(ir);
    let eq = gen::eq(ir);
    let ord = gen::ord(ir);

    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();
    let packed_ident = ir.ident_packed();

    output.append_all(quote! {
        #(#unpacked)*

        #pack

        #packed

        #unpack

        impl #generics_impl #packed_ident #generics_ty #generics_where {
            #pre

            #new

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

fn unpack_inner(input: &syn::DeriveInput) -> darling::Result<TokenStream> {
    // Generate unpacked type
    let mut stream = TokenStream::new();

    match &input.data {
        syn::Data::Union(_) => unimplemented!(),
        syn::Data::Struct(_) => (),
        syn::Data::Enum(r#enum) => {
            for variant in &r#enum.variants {
                match variant.fields.as_shape() {
                    // Assume wrapped type implements `ribbit::Pack`
                    Shape::Newtype => (),

                    // No generation needed
                    Shape::Unit => (),

                    // Generate struct for variant
                    Shape::Named | Shape::Tuple => {
                        let child = syn::DeriveInput {
                            attrs: variant.attrs.clone(),
                            vis: input.vis.clone(),
                            ident: variant.ident.clone(),
                            generics: input.generics.clone(),
                            data: syn::Data::Struct(syn::DataStruct {
                                struct_token: Default::default(),
                                fields: variant.fields.clone(),
                                semi_token: Default::default(),
                            }),
                        };

                        stream.append_all(unpack_item(child));
                    }
                }
            }
        }
    }

    stream.append_all(unpack_item(input.clone()));
    Ok(stream)
}

fn unpack_item(mut input: syn::DeriveInput) -> TokenStream {
    strip(&mut input.attrs);

    // It's possible to interact with packed structs only via
    // getters and setters, without eagerly unpacking the
    // entire type.
    input.attrs.push(parse_quote!(#[allow(dead_code)]));

    let (_, ty, _) = input.generics.split_for_impl();

    match &mut input.data {
        syn::Data::Enum(r#enum) => r#enum
            .variants
            .iter_mut()
            .flat_map(|variant| {
                strip(&mut variant.attrs);

                match variant.as_shape() {
                    Shape::Unit | Shape::Newtype => (),
                    Shape::Named | Shape::Tuple => {
                        let ident = variant.ident.clone();
                        let mut fields = syn::punctuated::Punctuated::new();
                        fields.push(syn::Field {
                            attrs: Vec::new(),
                            vis: syn::Visibility::Inherited,
                            mutability: syn::FieldMutability::None,
                            ident: None,
                            colon_token: Default::default(),
                            ty: parse_quote!(#ident #ty),
                        });
                        variant.fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                            paren_token: Default::default(),
                            unnamed: fields,
                        });
                    }
                }

                &mut variant.fields
            })
            .for_each(|field| strip(&mut field.attrs)),
        syn::Data::Struct(r#struct) => r#struct
            .fields
            .iter_mut()
            .for_each(|field| strip(&mut field.attrs)),
        syn::Data::Union(_) => unimplemented!(),
    }

    input.to_token_stream()
}

fn strip(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| match attr.path().segments.first() {
        None => true,
        Some(segment) => segment.ident != "ribbit",
    })
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
