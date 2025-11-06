use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::r#type::Arbitrary;
use crate::r#type::Loose;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tight {
    Unit,
    PhantomData,
    Bool,
    Loose { signed: bool, loose: Loose },
    Arbitrary(Arbitrary),
    NonZero { signed: bool, loose: Loose },
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
        let ident = match path.path.segments.last()? {
            segment if segment.ident == "PhantomData" => return Some(Self::PhantomData),
            segment if !segment.arguments.is_none() => return None,
            segment if segment.ident == "bool" => return Some(Self::Bool),
            segment => segment.ident.to_string(),
        };

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
            let loose = loose.ok_or(crate::Error::ArbitraryNonZero)?;
            return Ok(Self::NonZero { signed, loose });
        }

        match loose {
            Some(loose) => Ok(Self::Loose { signed, loose }),
            None => Arbitrary::new(signed, size).map(Self::Arbitrary),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tight::Unit | Tight::PhantomData => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.size(),
            Tight::Arbitrary(arbitrary) => arbitrary.size(),
            Tight::NonZero { signed: _, loose } => loose.size(),
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Tight::Unit | Tight::PhantomData => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.mask(),
            Tight::Arbitrary(arbitrary) => arbitrary.mask(),
            Tight::NonZero { signed: _, loose } => loose.mask(),
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
            Tight::Unit | Tight::PhantomData | Tight::Bool => Loose::N8,
            Tight::Loose { loose, .. } | Tight::NonZero { signed: _, loose } => loose,
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

            Tight::NonZero {
                signed: false,
                loose: _,
            } => quote!(#expression.get()),

            Tight::NonZero {
                signed: true,
                loose,
            } => quote!((#expression.get() as #loose)),
        }
    }

    pub(crate) fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Tight::Unit => quote!(()),
            Tight::PhantomData => quote!(::ribbit::private::PhantomData),
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
            Tight::NonZero { .. } | Tight::Arbitrary(_) => {
                quote!(unsafe { ::ribbit::convert::loose_to_packed::<#self>(#expression) })
            }
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
            Tight::Unit => return quote!(()).to_tokens(tokens),
            Tight::PhantomData => return quote!(::ribbit::private::PhantomData).to_tokens(tokens),
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
            Tight::NonZero { signed, loose } => {
                let signed = match signed {
                    true => 'I',
                    false => 'U',
                };
                let size = loose.size();
                format_ident!("NonZero{}{}", signed, size).to_token_stream()
            }
            Tight::Arbitrary(arbitrary) => return arbitrary.to_tokens(tokens),
        };

        quote!(::ribbit::private::#path).to_tokens(tokens)
    }
}

impl Display for Tight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tight::Unit => "()".fmt(f),
            Tight::PhantomData => "PhantomData".fmt(f),
            Tight::Bool => "bool".fmt(f),
            Tight::Loose {
                signed: true,
                loose: _,
            } => todo!(),
            Tight::Loose {
                signed: false,
                loose,
            } => loose.fmt(f),
            Tight::NonZero { signed, loose } => {
                let signed = match signed {
                    true => 'I',
                    false => 'U',
                };
                let size = loose.size();
                write!(f, "NonZero{signed}{size}")
            }
            Tight::Arbitrary(arbitrary) => arbitrary.fmt(f),
        }
    }
}
