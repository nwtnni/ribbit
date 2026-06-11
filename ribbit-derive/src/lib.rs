mod error;
mod gen;
mod input;
mod ir;
mod lift;
mod r#type;

pub(crate) use error::Error;
pub(crate) use r#type::Type;

use darling::FromDeriveInput as _;
use heck::ToSnakeCase as _;
use ir::Ir;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
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

    let precondition = gen::precondition(&ir);
    let new = gen::new(&ir);
    let pack = gen::pack(&ir);
    let packed = gen::packed(&ir);
    let unpack = gen::unpack(&ir);
    let get = gen::get(&ir);
    let with = gen::with(&ir);
    let into_raw = gen::into_raw(&ir);
    let from_raw_unchecked = gen::from_raw_unchecked(&ir);
    let from = gen::from(&ir);
    let debug = gen::debug(&ir);
    let hash = gen::hash(&ir);
    let eq = gen::eq(&ir);
    let ord = gen::ord(&ir);

    let generics = ir.generics_bounded();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();
    let ident_unpacked = ir.ident_unpacked();
    let ident_packed = ir.ident_packed();
    let ident_module = format_ident!("{}", ir.ident_packed().to_string().to_snake_case());
    let vis_packed = ir.opt().packed.vis(&input.vis);

    output.append_all(quote! {
        #pack

        mod #ident_module {
            use super::#ident_unpacked;

            #packed

            #unpack

            impl #generics_impl #ident_packed #generics_type #generics_where {
                #precondition

                #(#new)*

                #into_raw

                #from_raw_unchecked

                #(#get)*

                #(#with)*
            }

            #from
            #debug

            #hash
            #eq
            #ord
        }

        #vis_packed use #ident_module::#ident_packed;

    });

    Ok(())
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
