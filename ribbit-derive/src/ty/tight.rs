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
    pub(crate) fn size(&self) -> Spanned<usize> {
        self.repr.map_ref(|repr| repr.size())
    }

    pub(crate) fn mask(&self) -> usize {
        self.repr.mask()
    }

    pub(crate) fn new(nonzero: Spanned<bool>, size: Spanned<usize>) -> Self {
        Self {
            nonzero,
            signed: false,
            repr: size.map_ref(|size| Repr::new(*size)),
        }
    }

    pub(crate) fn is_loose(&self) -> bool {
        !*self.nonzero && !self.signed && matches!(&*self.repr, Repr::Loose(_))
    }

    pub(crate) fn loosen(self) -> Loose {
        match *self.repr {
            Repr::Unit => unreachable!("Should never loosen zero-sized type"),
            Repr::Bool => Loose::N8,
            Repr::Loose(loose) => loose,
            Repr::Arbitrary(arbitrary) => arbitrary.loosen(),
        }
    }

    pub(crate) fn parse(syn::TypePath { path, .. }: &syn::TypePath) -> Option<Self> {
        let segment = match path.segments.first()? {
            segment if path.segments.len() > 1 || !segment.arguments.is_none() => return None,
            segment => segment,
        };

        let ident = segment.ident.to_string();
        if ident == "bool" {
            return Some(Tight {
                nonzero: false.into(),
                signed: false,
                repr: Spanned::new(Repr::Bool, ident.span()),
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
            repr: Spanned::new(Repr::new(size), path.span()),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Repr {
    Unit,
    Bool,
    Loose(Loose),
    Arbitrary(Arbitrary),
}

impl Repr {
    fn new(size: usize) -> Self {
        match size {
            0 => Repr::Unit,
            8 => Repr::Loose(Loose::N8),
            16 => Repr::Loose(Loose::N16),
            32 => Repr::Loose(Loose::N32),
            64 => Repr::Loose(Loose::N64),
            size => Repr::Arbitrary(Arbitrary::new(size)),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Repr::Unit => 0,
            Repr::Bool => 1,
            Repr::Loose(loose) => loose.size(),
            Repr::Arbitrary(arbitrary) => arbitrary.size(),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Repr::Unit => 0,
            Repr::Bool => 1,
            Repr::Loose(loose) => loose.mask(),
            Repr::Arbitrary(arbitrary) => arbitrary.mask(),
        }
    }
}

impl ToTokens for Tight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = match (*self.nonzero, self.signed, *self.repr) {
            (_, _, Repr::Unit) => quote!(()),
            (_, true, _) => todo!(),

            (_, _, Repr::Bool) => quote!(bool),

            (true, _, Repr::Loose(Loose::N8)) => quote!(NonZeroU8),
            (true, _, Repr::Loose(Loose::N16)) => quote!(NonZeroU16),
            (true, _, Repr::Loose(Loose::N32)) => quote!(NonZeroU32),
            (true, _, Repr::Loose(Loose::N64)) => quote!(NonZeroU64),

            (false, _, Repr::Loose(Loose::N8)) => quote!(u8),
            (false, _, Repr::Loose(Loose::N16)) => quote!(u16),
            (false, _, Repr::Loose(Loose::N32)) => quote!(u32),
            (false, _, Repr::Loose(Loose::N64)) => quote!(u64),

            (true, _, Repr::Arbitrary(_)) => todo!(),
            (false, _, Repr::Arbitrary(arbitrary)) => {
                format_ident!("u{}", arbitrary.size()).to_token_stream()
            }
        };

        quote!(::ribbit::private::#repr).to_tokens(tokens)
    }
}
