use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::r#type::Arbitrary;
use crate::r#type::Loose;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tight {
    Unit,
    PhantomData,
    Bool,
    Arbitrary(Arbitrary),
}

impl Tight {
    pub(crate) fn from_size(non_zero: bool, size: usize) -> Result<Self, crate::Error> {
        Self::new(non_zero, false, size)
    }

    pub(crate) fn from_path(path: &syn::TypePath) -> Option<Self> {
        let ident = match path.path.segments.last()? {
            segment if segment.ident == "PhantomData" => return Some(Self::PhantomData),
            segment if !segment.arguments.is_none() => return None,
            segment if segment.ident == "bool" => return Some(Self::Bool),
            segment => segment.ident.to_string(),
        };

        let non_zero = ident.starts_with("NonZero");
        let signed = match non_zero {
            false => match &ident[..1] {
                "u" => false,
                "i" => true,
                _ => return None,
            },
            true => match &ident["NonZero".len()..][..1] {
                "U" => false,
                "I" => true,
                _ => return None,
            },
        };

        let size = ident[1 + match non_zero {
            false => 0,
            true => "NonZero".len(),
        }..]
            .parse::<usize>()
            .ok()?;

        Self::new(non_zero, signed, size).ok()
    }

    fn new(non_zero: bool, signed: bool, size: usize) -> Result<Self, crate::Error> {
        if size == 0 {
            return Ok(Self::Unit);
        }

        Arbitrary::new(signed, non_zero, size).map(Self::Arbitrary)
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tight::Unit | Tight::PhantomData => 0,
            Tight::Bool => 1,
            Tight::Arbitrary(arbitrary) => arbitrary.size(),
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Tight::Unit | Tight::PhantomData => 0,
            Tight::Bool => 1,
            Tight::Arbitrary(arbitrary) => arbitrary.mask(),
        }
    }

    pub(crate) fn is_non_zero(&self) -> bool {
        matches!(self, Self::Arbitrary(arbitrary) if arbitrary.is_non_zero())
    }

    pub(crate) fn is_loose(&self) -> bool {
        matches!(self, Self::Arbitrary(arbitrary) if arbitrary.is_loose())
    }

    pub(crate) fn to_loose(self) -> Loose {
        match self {
            Tight::Unit | Tight::PhantomData | Tight::Bool => Loose::N8,
            Tight::Arbitrary(arbitrary) => arbitrary.to_loose(),
        }
    }

    pub(crate) fn convert_to_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tight::Unit | Tight::PhantomData => {
                proc_macro2::Literal::usize_unsuffixed(0).to_token_stream()
            }
            Tight::Bool => {
                let zero = proc_macro2::Literal::usize_unsuffixed(0);
                let one = proc_macro2::Literal::usize_unsuffixed(1);
                quote! {
                    match #expression {
                        false => #zero,
                        true => #one,
                    }
                }
            }
            Tight::Arbitrary(arbitrary) => arbitrary.convert_to_loose(expression),
        }
    }

    pub(crate) fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tight::Unit => quote!(()),
            Tight::PhantomData => quote!(::ribbit::PhantomData),
            Tight::Bool => {
                let zero = proc_macro2::Literal::usize_unsuffixed(0);
                quote!((#expression != #zero))
            }
            Tight::Arbitrary(arbitrary) => arbitrary.convert_from_loose(expression),
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
            Tight::Unit => return quote!(()).to_tokens(tokens),
            Tight::PhantomData => return quote!(::ribbit::PhantomData).to_tokens(tokens),
            Tight::Bool => quote!(bool),
            Tight::Arbitrary(arbitrary) => return arbitrary.to_tokens(tokens),
        };

        quote!(::ribbit::#path).to_tokens(tokens)
    }
}

impl Display for Tight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tight::Unit => "()".fmt(f),
            Tight::PhantomData => "PhantomData".fmt(f),
            Tight::Bool => "bool".fmt(f),
            Tight::Arbitrary(arbitrary) => arbitrary.fmt(f),
        }
    }
}
