use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::repr::Arbitrary;
use crate::repr::Native;
use crate::Spanned;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Leaf {
    pub(crate) nonzero: Spanned<bool>,
    pub(crate) signed: bool,
    pub(crate) repr: Repr,
}

impl PartialEq for Leaf {
    fn eq(&self, other: &Self) -> bool {
        (*self.nonzero).eq(&*other.nonzero)
            && self.signed == other.signed
            && self.repr == other.repr
    }
}

impl Eq for Leaf {}

impl Leaf {
    pub(crate) fn size(&self) -> usize {
        self.repr.size()
    }

    pub(crate) fn mask(&self) -> usize {
        self.repr.mask()
    }

    pub(crate) fn new(nonzero: Spanned<bool>, size: usize) -> Self {
        Self {
            nonzero,
            signed: false,
            repr: Repr::new(size),
        }
    }

    pub(crate) fn as_native(&self) -> Native {
        match self.repr {
            Repr::Native(native) => native,
            Repr::Arbitrary(arbitrary) => arbitrary.as_native(),
        }
    }

    pub(crate) fn from_path(syn::TypePath { qself, path }: &syn::TypePath) -> Option<Self> {
        if qself.is_some() {
            todo!();
        }

        if path.leading_colon.is_some() {
            todo!()
        }

        if path.segments.len() > 1 {
            todo!();
        }

        let segment = path.segments.first().unwrap();

        if !segment.arguments.is_none() {
            todo!();
        }

        let ident = segment.ident.to_string();

        if !ident.is_ascii() {
            todo!();
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

        Some(Leaf {
            nonzero: nonzero.into(),
            signed,
            repr: Repr::new(size),
        })
    }
}

impl From<Native> for Leaf {
    fn from(native: Native) -> Self {
        Leaf {
            nonzero: false.into(),
            signed: false,
            repr: Repr::Native(native),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Repr {
    Native(Native),
    Arbitrary(Arbitrary),
}

impl Repr {
    fn new(size: usize) -> Self {
        match size {
            8 => Repr::Native(Native::N8),
            16 => Repr::Native(Native::N16),
            32 => Repr::Native(Native::N32),
            64 => Repr::Native(Native::N64),
            size => Repr::Arbitrary(Arbitrary::new(size)),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Repr::Native(native) => native.size(),
            Repr::Arbitrary(arbitrary) => arbitrary.size(),
        }
    }

    pub(crate) fn mask(&self) -> usize {
        match self {
            Repr::Native(native) => native.mask(),
            Repr::Arbitrary(arbitrary) => arbitrary.mask(),
        }
    }
}

impl ToTokens for Leaf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = match (*self.nonzero, self.signed, self.repr) {
            (_, true, _) => todo!(),

            (true, _, Repr::Native(Native::N8)) => quote!(NonZeroU8),
            (true, _, Repr::Native(Native::N16)) => quote!(NonZeroU16),
            (true, _, Repr::Native(Native::N32)) => quote!(NonZeroU32),
            (true, _, Repr::Native(Native::N64)) => quote!(NonZeroU64),

            (false, _, Repr::Native(Native::N8)) => quote!(u8),
            (false, _, Repr::Native(Native::N16)) => quote!(u16),
            (false, _, Repr::Native(Native::N32)) => quote!(u32),
            (false, _, Repr::Native(Native::N64)) => quote!(u64),

            (true, _, Repr::Arbitrary(_)) => todo!(),
            (false, _, Repr::Arbitrary(arbitrary)) => {
                format_ident!("u{}", arbitrary.size()).to_token_stream()
            }
        };

        quote!(::ribbit::private::#repr).to_tokens(tokens)
    }
}
