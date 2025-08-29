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
        match size {
            0 => return Ok(Self::Unit),
            1 => return Ok(Self::Bool),
            _ => (),
        }

        let loose = Loose::new(size);

        if nonzero {
            assert!(!signed, "[INTERNAL ERROR]: nonzero signed tight type");
            let loose = loose.ok_or(crate::Error::ArbitraryNonZero)?;
            return Ok(Self::NonZero(loose));
        }

        match loose {
            Some(loose) => Ok(Self::Loose {
                signed: false,
                loose,
            }),
            None => Arbitrary::new(size).map(Self::Arbitrary),
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

    pub(crate) fn is_loose(&self) -> bool {
        matches!(self, Self::Loose { .. })
    }

    pub(crate) fn is_nonzero(&self) -> bool {
        matches!(self, Self::NonZero { .. })
    }

    pub(crate) fn loosen(&self) -> &Loose {
        match self {
            Tight::Unit | Tight::Bool => &Loose::N8,
            Tight::Loose { loose, .. } | Tight::NonZero(loose) => loose,
            Tight::Arbitrary(arbitrary) => arbitrary.loosen(),
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
