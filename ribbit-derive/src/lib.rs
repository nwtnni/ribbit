mod get;
mod input;
mod ir;

use darling::FromDeriveInput as _;
use proc_macro2::TokenStream;
use quote::quote;
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

    let ir = ir::new(&attr, &input, &item);
    Ok(Output { ir }.to_token_stream())
}

struct Output<'input> {
    ir: ir::Struct<'input>,
}

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ir = &self.ir;
        let get = crate::get::Struct::new(ir);

        let output = quote! {
            #ir

            #get
        };

        output.to_tokens(tokens);
    }
}
