use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;
use crate::ty::Loose;

#[derive(Clone, Debug)]
pub(crate) enum Tight {
    Unit,
    Bool,
    Loose {
        signed: bool,
        loose: Loose,
    },
    Arbitrary {
        path: Option<syn::TypePath>,
        inner: Arbitrary,
    },
    NonZero {
        path: Option<syn::TypePath>,
        loose: Loose,
    },
}

impl PartialEq for Tight {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Bool, Self::Bool) => true,
            (
                Self::Loose {
                    signed: signed_l,
                    loose: loose_l,
                },
                Self::Loose {
                    signed: signed_r,
                    loose: loose_r,
                },
            ) => signed_l == signed_r && loose_l == loose_r,
            (
                Self::Arbitrary {
                    inner: inner_l,
                    path: _,
                },
                Self::Arbitrary {
                    inner: inner_r,
                    path: _,
                },
            ) => inner_l == inner_r,
            (
                Self::NonZero {
                    loose: inner_l,
                    path: _,
                },
                Self::NonZero {
                    loose: inner_r,
                    path: _,
                },
            ) => inner_l == inner_r,
            _ => false,
        }
    }
}

impl Eq for Tight {}

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
        Self::new(None, nonzero, false, size)
    }

    pub(crate) fn from_path(path: &syn::TypePath) -> Option<Result<Self, crate::Error>> {
        let segment = match path.path.segments.last()? {
            segment if !segment.arguments.is_none() => return None,
            segment => segment,
        };

        let ident = segment.ident.to_string();
        if ident == "bool" {
            return Some(Ok(Self::Bool));
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

        Some(Self::new(Some(path.clone()), nonzero, signed, size))
    }

    fn new(
        path: Option<syn::TypePath>,
        nonzero: bool,
        signed: bool,
        size: usize,
    ) -> Result<Self, crate::Error> {
        match size {
            0 => return Ok(Self::Unit),
            1 if path.is_none() => return Ok(Self::Bool),
            _ => (),
        }

        let loose = Loose::new(size);

        if nonzero {
            assert!(
                !signed,
                "[INTERNAL ERROR]: signed nonzero types are unsupported"
            );
            let loose = loose.ok_or(crate::Error::ArbitraryNonZero)?;
            return Ok(Self::NonZero { path, loose });
        }

        match loose {
            Some(loose) => Ok(Self::Loose { signed, loose }),
            None => Arbitrary::new(size).map(|inner| Self::Arbitrary { path, inner }),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tight::Unit => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.size(),
            Tight::Arbitrary { inner, path: _ } => inner.size(),
            Tight::NonZero {
                loose: inner,
                path: _,
            } => inner.size(),
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Tight::Unit => 0,
            Tight::Bool => 1,
            Tight::Loose { signed: _, loose } => loose.mask(),
            Tight::Arbitrary { inner, path: _ } => inner.mask(),
            Tight::NonZero {
                loose: inner,
                path: _,
            } => inner.mask(),
        }
    }

    pub(crate) fn is_loose(&self) -> bool {
        matches!(self, Self::Loose { .. })
    }

    pub(crate) fn is_nonzero(&self) -> bool {
        matches!(self, Self::NonZero { .. })
    }

    pub(crate) fn loosen(&self) -> Loose {
        match self {
            Tight::Unit | Tight::Bool => Loose::N8,
            Tight::Loose { signed: _, loose } => *loose,
            Tight::Arbitrary { inner, path: _ } => inner.loosen(),
            Tight::NonZero {
                loose: inner,
                path: _,
            } => *inner,
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
                loose: _,
            } => todo!(),

            Tight::Loose {
                signed: false,
                loose,
            } => return loose.to_tokens(tokens),

            Tight::Arbitrary {
                inner: _,
                path: Some(path),
            }
            | Tight::NonZero {
                loose: _,
                path: Some(path),
            } => return path.to_tokens(tokens),

            Tight::NonZero {
                loose: Loose::N8,
                path: None,
            } => quote!(NonZeroU8),
            Tight::NonZero {
                loose: Loose::N16,
                path: None,
            } => quote!(NonZeroU16),
            Tight::NonZero {
                loose: Loose::N32,
                path: None,
            } => quote!(NonZeroU32),
            Tight::NonZero {
                loose: Loose::N64,
                path: None,
            } => quote!(NonZeroU64),
            Tight::NonZero {
                loose: Loose::N128,
                path: None,
            } => quote!(NonZeroU128),

            Tight::Arbitrary { inner, path: None } => return inner.to_tokens(tokens),
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

            Tight::Arbitrary {
                inner: _,
                path: Some(path),
            }
            | Tight::NonZero {
                loose: _,
                path: Some(path),
            } => write!(f, "{}", path.to_token_stream()),

            Tight::NonZero {
                loose: Loose::N8,
                path: None,
            } => "NonZeroU8".fmt(f),
            Tight::NonZero {
                loose: Loose::N16,
                path: None,
            } => "NonZeroU16".fmt(f),
            Tight::NonZero {
                loose: Loose::N32,
                path: None,
            } => "NonZeroU32".fmt(f),
            Tight::NonZero {
                loose: Loose::N64,
                path: None,
            } => "NonZeroU64".fmt(f),
            Tight::NonZero {
                loose: Loose::N128,
                path: None,
            } => "NonZeroU128".fmt(f),

            Tight::Arbitrary { inner, path: None } => inner.fmt(f),
        }
    }
}
