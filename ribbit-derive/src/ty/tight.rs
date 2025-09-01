use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;
use crate::ty::Loose;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tight {
    Unit,
    Bool,
    Loose { signed: bool, loose: Loose },
    Arbitrary(Arbitrary),
    NonZero(Loose),
}

impl From<Loose> for Tight {
    fn from(loose: Loose) -> Self {
        Self::Loose {
            signed: false,
            loose,
        }
    }
}

impl Tight {
    pub(crate) fn from_size(nonzero: bool, size: usize) -> Result<Self, crate::Error> {
        Self::new(nonzero, false, size)
    }

    pub(crate) fn from_path(path: &syn::TypePath) -> Option<Self> {
        let segment = match path.path.segments.last()? {
            segment if !segment.arguments.is_none() => return None,
            segment => segment,
        };

        let ident = segment.ident.to_string();
        if ident == "bool" {
            return Some(Self::Bool);
        }

        let nonzero = ident.starts_with("NonZero");
        let signed = match nonzero {
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

        let size = ident[1 + match nonzero {
            false => 0,
            true => "NonZero".len(),
        }..]
            .parse::<usize>()
            .ok()?;

        Self::new(nonzero, signed, size).ok()
    }

    fn new(nonzero: bool, signed: bool, size: usize) -> Result<Self, crate::Error> {
        if size == 0 {
            return Ok(Self::Unit);
        }

        let loose = Loose::new(size);

        if nonzero {
            assert!(!signed, "[INTERNAL ERROR]: nonzero signed tight type");
            let loose = loose.ok_or(crate::Error::ArbitraryNonZero)?;
            return Ok(Self::NonZero(loose));
        }

        match loose {
            Some(loose) => Ok(Self::Loose { signed, loose }),
            None => Arbitrary::new(signed, size).map(Self::Arbitrary),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tight::Unit => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.size(),
            Tight::Arbitrary(arbitrary) => arbitrary.size(),
            Tight::NonZero(loose) => loose.size(),
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Tight::Unit => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.mask(),
            Tight::Arbitrary(arbitrary) => arbitrary.mask(),
            Tight::NonZero(loose) => loose.mask(),
        }
    }

    pub(crate) fn is_nonzero(&self) -> bool {
        matches!(self, Self::NonZero { .. })
    }

    pub(crate) fn is_loose(&self) -> bool {
        matches!(self, Self::Loose { .. })
    }

    pub(crate) fn to_loose(self) -> Loose {
        match self {
            Tight::Unit | Tight::Bool => Loose::N8,
            Tight::Loose { loose, .. } | Tight::NonZero(loose) => loose,
            Tight::Arbitrary(arbitrary) => arbitrary.to_loose(),
        }
    }

    pub(crate) fn convert_to_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tight::Unit => proc_macro2::Literal::usize_unsuffixed(0).to_token_stream(),
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
            Tight::Loose {
                signed: false,
                loose: _,
            } => expression,
            Tight::Loose {
                signed: true,
                loose,
            } => quote!((#expression as #loose)),

            Tight::Arbitrary(arbitrary) if arbitrary.is_signed() => {
                let loose = arbitrary.to_loose();
                quote!((#expression.value() as #loose))
            }
            Tight::Arbitrary(_) => quote!(#expression.value()),

            Tight::NonZero(_) => quote!(#expression.get()),
        }
    }

    pub(crate) fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tight::Unit => quote!(()),
            Tight::Bool => {
                let zero = proc_macro2::Literal::usize_unsuffixed(0);
                quote!((#expression != #zero))
            }
            Tight::Loose { signed: false, .. } => expression,
            Tight::Loose {
                signed: true,
                loose: _,
            } => quote!((#expression as #self)),

            // Skip validation logic in `NonZero` and `Arbitrary` constructors
            Tight::NonZero(_) | Tight::Arbitrary(_) => {
                quote!(unsafe { ::ribbit::convert::loose_to_packed::<#self>(#expression) })
            }
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
            Tight::Unit => return quote!(()).to_tokens(tokens),
            Tight::Bool => quote!(bool),
            Tight::Loose {
                signed: true,
                loose,
            } => {
                return match loose {
                    Loose::N8 => quote!(i8),
                    Loose::N16 => quote!(i8),
                    Loose::N32 => quote!(i32),
                    Loose::N64 => quote!(i64),
                    Loose::N128 => quote!(i128),
                }
                .to_tokens(tokens)
            }
            Tight::Loose {
                signed: false,
                loose,
            } => return loose.to_tokens(tokens),
            Tight::NonZero(Loose::N8) => quote!(NonZeroU8),
            Tight::NonZero(Loose::N16) => quote!(NonZeroU16),
            Tight::NonZero(Loose::N32) => quote!(NonZeroU32),
            Tight::NonZero(Loose::N64) => quote!(NonZeroU64),
            Tight::NonZero(Loose::N128) => quote!(NonZeroU128),
            Tight::Arbitrary(arbitrary) => return arbitrary.to_tokens(tokens),
        };

        quote!(::ribbit::private::#path).to_tokens(tokens)
    }
}

impl Display for Tight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tight::Unit => "()".fmt(f),
            Tight::Bool => "bool".fmt(f),
            Tight::Loose {
                signed: true,
                loose: _,
            } => todo!(),
            Tight::Loose {
                signed: false,
                loose,
            } => loose.fmt(f),
            Tight::NonZero(Loose::N8) => "NonZeroU8".fmt(f),
            Tight::NonZero(Loose::N16) => "NonZeroU16".fmt(f),
            Tight::NonZero(Loose::N32) => "NonZeroU32".fmt(f),
            Tight::NonZero(Loose::N64) => "NonZeroU64".fmt(f),
            Tight::NonZero(Loose::N128) => "NonZeroU128".fmt(f),
            Tight::Arbitrary(arbitrary) => arbitrary.fmt(f),
        }
    }
}
