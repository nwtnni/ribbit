use core::fmt::Display;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::r#type::Loose;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Arbitrary {
    signed: bool,
    non_zero: bool,
    size: usize,
}

impl Arbitrary {
    pub(super) const N8: Self = Self {
        signed: false,
        non_zero: false,
        size: 8,
    };
    pub(super) const N16: Self = Self {
        signed: false,
        non_zero: false,
        size: 16,
    };
    pub(super) const N32: Self = Self {
        signed: false,
        non_zero: false,
        size: 32,
    };
    pub(super) const N64: Self = Self {
        signed: false,
        non_zero: false,
        size: 64,
    };
    pub(super) const N128: Self = Self {
        signed: false,
        non_zero: false,
        size: 128,
    };

    pub(super) const fn new(
        signed: bool,
        non_zero: bool,
        size: usize,
    ) -> Result<Self, crate::Error> {
        match size {
            129.. => Err(crate::Error::ArbitrarySize { size }),
            _ => Ok(Self {
                signed,
                non_zero,
                size,
            }),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn is_non_zero(&self) -> bool {
        self.non_zero
    }

    pub(crate) fn mask(&self) -> u128 {
        crate::mask(self.size)
    }

    pub(crate) fn is_loose(self) -> bool {
        Loose::new(self.size).is_some()
    }

    pub(crate) fn to_loose(self) -> Loose {
        match self.size {
            0..=8 => Loose::N8,
            9..=16 => Loose::N16,
            17..=32 => Loose::N32,
            33..=64 => Loose::N64,
            65..=128 => Loose::N128,
            _ => unreachable!(),
        }
    }

    pub(crate) fn convert_to_loose(&self, expression: TokenStream) -> TokenStream {
        if self.non_zero {
            return if self.signed {
                let loose = self.to_loose();
                quote!((#expression.get() as #loose))
            } else {
                quote!(#expression.get())
            };
        }

        if let Some(loose) = Loose::new(self.size) {
            return if self.signed {
                quote!((#expression as #loose))
            } else {
                expression
            };
        }

        if self.signed {
            let loose = self.to_loose();
            quote!((#expression.value() as #loose))
        } else {
            quote!(#expression.value())
        }
    }

    pub(crate) fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        if self.non_zero || !self.is_loose() {
            // Skip validation in non-zero and arbitrary-int constructors
            return quote!(unsafe { ::ribbit::convert::loose_to_packed::<#self>(#expression) });
        };

        if self.signed {
            quote!((#expression as #self))
        } else {
            expression
        }
    }
}

impl ToTokens for Arbitrary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = if self.non_zero {
            let signed = match self.signed {
                true => 'I',
                false => 'U',
            };
            let size = self.to_loose().size();
            format_ident!("NonZero{}{}", signed, size)
        } else {
            let signed = match self.signed {
                true => 'i',
                false => 'u',
            };
            let size = self.size();
            format_ident!("{}{}", signed, size)
        };

        quote!(::ribbit::#ident).to_tokens(tokens)
    }
}

impl Display for Arbitrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.non_zero {
            let signed = match self.signed {
                true => 'I',
                false => 'U',
            };
            let size = self.to_loose().size();
            write!(f, "NonZero{}{}", signed, size)
        } else {
            let signed = match self.signed {
                true => 'i',
                false => 'u',
            };
            let size = self.size();
            write!(f, "{}{}", signed, size)
        }
    }
}
