use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::ty::Arbitrary;
use crate::ty::Loose;
use crate::Spanned;

#[derive(Copy, Clone, Debug)]
pub struct Tight {
    pub(crate) nonzero: Spanned<bool>,
    pub(crate) signed: bool,
    pub(crate) repr: Spanned<Repr>,
}

impl PartialEq for Tight {
    fn eq(&self, other: &Self) -> bool {
        *self.nonzero == *other.nonzero && self.signed == other.signed && *self.repr == *other.repr
    }
}

impl Eq for Tight {}

impl From<Loose> for Tight {
    fn from(loose: Loose) -> Self {
        Self {
            nonzero: Spanned::from(false),
            signed: false,
            repr: Spanned::from(Repr::Loose(loose)),
        }
    }
}

impl Tight {
    pub(crate) fn from_size(nonzero: Spanned<bool>, size: Spanned<usize>) -> Self {
        Self {
            nonzero,
            signed: false,
            repr: size.map_ref(|size| Repr::from_size(*size)),
        }
    }

    pub(crate) fn from_path(syn::TypePath { path, .. }: &syn::TypePath) -> Option<Self> {
        let segment = match path.segments.first()? {
            segment if path.segments.len() > 1 || !segment.arguments.is_none() => return None,
            segment => segment,
        };

        let ident = segment.ident.to_string();
        if ident == "bool" {
            return Some(Tight {
                nonzero: false.into(),
                signed: false,
                repr: Spanned::new(Repr::Loose(Loose::Bool), ident.span()),
            });
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

        Some(Tight {
            nonzero: nonzero.into(),
            signed,
            repr: Spanned::new(Repr::from_path(size), path.span()),
        })
    }

    pub(crate) fn size(&self) -> Spanned<usize> {
        self.repr.map_ref(|repr| repr.size())
    }

    pub(crate) fn mask(&self) -> usize {
        self.repr.mask()
    }

    pub(crate) fn is_loose(&self) -> bool {
        !*self.nonzero && !self.signed && matches!(&*self.repr, Repr::Loose(_))
    }

    pub(crate) fn loosen(self) -> Loose {
        match *self.repr {
            Repr::Loose(loose) => loose,
            Repr::Arbitrary(arbitrary) => arbitrary.loosen(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Repr {
    Loose(Loose),
    Arbitrary(Arbitrary),
}

impl Repr {
    /// The canonical internal representation for a 1-bit
    /// type is `bool`, but the user can opt into u1 by
    /// using the type explicitly.
    fn from_size(size: usize) -> Self {
        match size {
            1 => Repr::Loose(Loose::Bool),
            _ => Self::from_path(size),
        }
    }

    fn from_path(size: usize) -> Self {
        match size {
            0 => Repr::Loose(Loose::Unit),
            8 => Repr::Loose(Loose::N8),
            16 => Repr::Loose(Loose::N16),
            32 => Repr::Loose(Loose::N32),
            64 => Repr::Loose(Loose::N64),
            size => Repr::Arbitrary(Arbitrary::new(size)),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Repr::Loose(loose) => loose.size(),
            Repr::Arbitrary(arbitrary) => arbitrary.size(),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Repr::Loose(loose) => loose.mask(),
            Repr::Arbitrary(arbitrary) => arbitrary.mask(),
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = match (*self.nonzero, self.signed, *self.repr) {
            (_, true, _) => todo!(),

            (true, _, Repr::Loose(Loose::N8)) => quote!(NonZeroU8),
            (true, _, Repr::Loose(Loose::N16)) => quote!(NonZeroU16),
            (true, _, Repr::Loose(Loose::N32)) => quote!(NonZeroU32),
            (true, _, Repr::Loose(Loose::N64)) => quote!(NonZeroU64),

            (_, _, Repr::Loose(loose)) => return loose.to_tokens(tokens),

            (true, _, Repr::Arbitrary(_)) => todo!(),
            (false, _, Repr::Arbitrary(arbitrary)) => {
                format_ident!("u{}", arbitrary.size()).to_token_stream()
            }
        };

        quote!(::ribbit::private::#repr).to_tokens(tokens)
    }
}

impl Display for Tight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match (*self.nonzero, self.signed, *self.repr) {
            (_, true, _) => todo!(),

            (true, _, Repr::Loose(Loose::N8)) => "NonZeroU8",
            (true, _, Repr::Loose(Loose::N16)) => "NonZeroU16",
            (true, _, Repr::Loose(Loose::N32)) => "NonZeroU32",
            (true, _, Repr::Loose(Loose::N64)) => "NonZeroU64",

            (_, _, Repr::Loose(Loose::Unit)) => "()",
            (_, _, Repr::Loose(Loose::Bool)) => "bool",
            (_, _, Repr::Loose(Loose::N8)) => "u8",
            (_, _, Repr::Loose(Loose::N16)) => "u16",
            (_, _, Repr::Loose(Loose::N32)) => "u32",
            (_, _, Repr::Loose(Loose::N64)) => "u64",

            (true, _, Repr::Arbitrary(_)) => todo!(),
            (false, _, Repr::Arbitrary(arbitrary)) => return write!(f, "u{}", arbitrary.size()),
        };

        name.fmt(f)
    }
}
