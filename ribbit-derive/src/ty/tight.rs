use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty::Arbitrary;
use crate::ty::Loose;

#[derive(Clone, Debug)]
pub(crate) enum Tight {
    Loose {
        signed: bool,
        loose: Loose,
    },
    Arbitrary {
        inner: Arbitrary,
        path: Option<syn::TypePath>,
    },
    NonZero {
        inner: Loose,
        path: Option<syn::TypePath>,
    },
}

impl PartialEq for Tight {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
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
                    inner: inner_l,
                    path: _,
                },
                Self::NonZero {
                    inner: inner_r,
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
            return Some(Ok(Tight::Loose {
                signed: false,
                loose: Loose::Bool,
            }));
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
        let loose = match &path {
            Some(_) => Loose::new_external,
            None => Loose::new_internal,
        };

        let tight = match nonzero {
            false => loose(size)
                .map(|loose| Self::Loose { signed, loose })
                .unwrap_or_else(|| Self::Arbitrary {
                    inner: Arbitrary::new(size),
                    path,
                }),
            true => loose(size)
                .ok_or(crate::Error::ArbitraryNonZero)
                .map(|loose| Self::NonZero { inner: loose, path })?,
        };

        Ok(tight)
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tight::Loose { signed: _, loose } => loose.size(),
            Tight::Arbitrary { inner, path: _ } => inner.size(),
            Tight::NonZero { inner, path: _ } => inner.size(),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Tight::Loose { signed: _, loose } => loose.mask(),
            Tight::Arbitrary { inner, path: _ } => inner.mask(),
            Tight::NonZero { inner, path: _ } => inner.mask(),
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
            Tight::Loose { signed: _, loose } => *loose,
            Tight::Arbitrary { inner, path: _ } => inner.loosen(),
            Tight::NonZero { inner, path: _ } => *inner,
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
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
                inner: _,
                path: Some(path),
            } => return path.to_tokens(tokens),

            Tight::NonZero {
                inner: Loose::N8,
                path: None,
            } => quote!(NonZeroU8),
            Tight::NonZero {
                inner: Loose::N16,
                path: None,
            } => quote!(NonZeroU16),
            Tight::NonZero {
                inner: Loose::N32,
                path: None,
            } => quote!(NonZeroU32),
            Tight::NonZero {
                inner: Loose::N64,
                path: None,
            } => quote!(NonZeroU64),
            Tight::NonZero {
                inner: Loose::N128,
                path: None,
            } => quote!(NonZeroU128),
            Tight::NonZero {
                inner: _,
                path: None,
            } => unreachable!(),

            Tight::Arbitrary { inner, path: None } => return inner.to_tokens(tokens),
        };

        quote!(::ribbit::private::#path).to_tokens(tokens)
    }
}

impl Display for Tight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                inner: _,
                path: Some(path),
            } => write!(f, "{}", path.to_token_stream()),

            Tight::NonZero {
                inner: Loose::N8,
                path: None,
            } => "NonZeroU8".fmt(f),
            Tight::NonZero {
                inner: Loose::N16,
                path: None,
            } => "NonZeroU16".fmt(f),
            Tight::NonZero {
                inner: Loose::N32,
                path: None,
            } => "NonZeroU32".fmt(f),
            Tight::NonZero {
                inner: Loose::N64,
                path: None,
            } => "NonZeroU64".fmt(f),
            Tight::NonZero {
                inner: Loose::N128,
                path: None,
            } => "NonZeroU128".fmt(f),
            Tight::NonZero {
                inner: _,
                path: None,
            } => unreachable!(),

            Tight::Arbitrary { inner, path: None } => inner.fmt(f),
        }
    }
}
