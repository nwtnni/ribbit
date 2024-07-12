mod input;

use darling::FromDeriveInput as _;
use proc_macro2::Span;
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
        .map(|output| output.to_token_stream())
        .unwrap_or_else(|error| error.write_errors())
        .into()
}

fn pack_inner(attr: TokenStream, input: syn::DeriveInput) -> Result<Output, darling::Error> {
    let attr = input::Attr::from_token_stream(attr)?;
    let item = input::Item::from_derive_input(&input)?;

    Ok(Output {
        item,
        attrs: input.attrs,
        size: attr.size,
    })
}

struct Output {
    item: input::Item,
    attrs: Vec<syn::Attribute>,
    size: u8,
}

impl ToTokens for Output {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.item.ident;
        let attrs = &self.attrs;

        let repr = syn::Ident::new(&format!("u{}", self.size), Span::call_site());
        let repr = match self.size {
            8 | 16 | 32 | 64 => quote!(#repr),
            _ => quote!(::ribbit::private::arbitrary_int::#repr),
        };

        let output = quote! {
            #( #attrs )*
            struct #ident {
                value: #repr,
            }
        };

        output.to_tokens(tokens);
    }
}
