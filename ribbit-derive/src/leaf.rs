use darling::util::SpannedValue;
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;

mod arbitrary;
mod native;

pub(crate) use arbitrary::Arbitrary;
pub(crate) use native::Native;

use crate::Spanned;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Leaf {
    pub(crate) nonzero: bool,
    pub(crate) signed: bool,
    pub(crate) repr: Repr,
}

impl Leaf {
    pub(crate) fn size(&self) -> usize {
        self.repr.size()
    }

    pub(crate) fn mask(&self) -> usize {
        self.repr.mask()
    }

    pub(crate) fn convert_to_native<T: ToTokens>(&self, input: T) -> TokenStream {
        match (self.nonzero, self.repr) {
            (true, Repr::Native(_)) => quote!(#input.get()),
            (false, Repr::Native(_)) => quote!(#input),
            (true, Repr::Arbitrary(_)) => todo!(),
            (false, Repr::Arbitrary(_)) => quote!(#input.value()),
        }
    }

    pub(crate) fn convert_from_native<T: ToTokens>(&self, input: T) -> TokenStream {
        match (self.nonzero, self.repr) {
            (true, Repr::Native(_)) => quote!(match #self::new(#input) {
                None => panic!(),
                Some(output) => output,
            }),
            (false, Repr::Native(_)) => quote!(#input),
            (true, Repr::Arbitrary(_)) => todo!(),
            (false, Repr::Arbitrary(arbitrary)) => {
                let mask = Literal::usize_unsuffixed(arbitrary.mask());
                quote!(#self::new(#input & #mask))
            }
        }
    }

    pub(crate) fn new(nonzero: bool, size: usize) -> Self {
        Self {
            nonzero,
            signed: false,
            repr: Repr::new(size),
        }
    }

    pub(crate) fn as_native(&self) -> Self {
        match self.repr {
            Repr::Native(_) => *self,
            Repr::Arbitrary(arbitrary) => Self {
                repr: Repr::Native(arbitrary.as_native()),
                ..*self
            },
        }
    }

    pub(crate) fn from_ty(ty: &syn::Type) -> Option<Spanned<Self>> {
        match ty {
            syn::Type::Array(_) => todo!(),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Path(path) => Self::from_path(path),
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(_) => todo!(),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }

    fn from_path(ty_path @ syn::TypePath { qself, path }: &syn::TypePath) -> Option<Spanned<Self>> {
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

        Some(
            SpannedValue::new(
                Leaf {
                    nonzero,
                    signed,
                    repr: Repr::new(size),
                },
                ty_path.span(),
            )
            .into(),
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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

    pub(crate) fn as_native(&self) -> Native {
        match self {
            Repr::Native(native) => *native,
            Repr::Arbitrary(arbitrary) => arbitrary.as_native(),
        }
    }
}

impl ToTokens for Leaf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = match (self.nonzero, self.signed, self.repr) {
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
