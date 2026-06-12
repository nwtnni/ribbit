mod arbitrary;
mod loose;
mod tight;

pub(crate) use arbitrary::Arbitrary;
use darling::usage::CollectTypeParams as _;
use darling::usage::IdentSet;
use darling::util::SpannedValue;
pub(crate) use loose::Loose;
use syn::TypePath;
pub(crate) use tight::Tight;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::error::bail;
use crate::ir;
use crate::Error;

#[derive(Clone, Debug, Eq)]
pub(crate) enum Type {
    Tight {
        tight: Tight,
    },
    User {
        path: TypePath,
        uses: IdentSet,
        tight: Tight,
    },
}

impl Type {
    pub(crate) fn parse(
        newtype: bool,
        opt_variant: &ir::VariantOpt,
        opt_field: &ir::FieldOpt,
        type_params: &IdentSet,
        ty: syn::Type,
    ) -> darling::Result<SpannedValue<Self>> {
        let syn::Type::Path(path) = ty else {
            bail!(ty=> Error::UnsupportedType)
        };

        let span = path.span();

        if let Some(tight) = Tight::from_path(&path) {
            if let Some(expected) = opt_field.size.filter(|size| *size != tight.size()) {
                bail!(span=> Error::WrongSize {
                    expected,
                    actual: tight.size(),
                    tight,
                });
            }

            return Ok(SpannedValue::new(Self::Tight { tight }, span));
        };

        // For convenience, forward non_zero and size annotations
        // for newtype structs.
        let non_zero = match (newtype, *opt_field.non_zero) {
            (false, non_zero) | (true, non_zero @ true) => non_zero,
            (true, false) => *opt_variant.non_zero,
        };
        let size = match (newtype, *opt_field.size) {
            (false, size) | (true, size @ Some(_)) => size,
            (true, None) => *opt_variant.size,
        };

        let Some(size) = size else {
            bail!(span=> Error::OpaqueSize);
        };

        let tight = Tight::from_size(non_zero, size);

        let tight = match tight {
            Ok(tight) => tight,
            Err(error) => bail!(span=> error),
        };

        let uses = std::iter::once(&path)
            .collect_type_params_cloned(&darling::usage::Purpose::Declare.into(), type_params);

        Ok(SpannedValue::new(Self::User { path, uses, tight }, span))
    }

    pub(crate) fn is_user(&self) -> bool {
        matches!(self, Self::User { .. })
    }

    pub(crate) fn is_generic(&self) -> bool {
        matches!(self, Self::User { uses, .. } if !uses.is_empty())
    }

    pub(crate) fn is_loose(&self) -> bool {
        matches!(self, Self::Tight { tight, .. } if tight.is_loose())
    }

    pub(crate) fn as_tight(&self) -> &Tight {
        match self {
            Self::Tight { tight, .. } | Self::User { tight, .. } => tight,
        }
    }

    pub(crate) fn to_loose(&self) -> Loose {
        self.as_tight().to_loose()
    }

    pub(crate) fn packed(&self) -> TokenStream {
        match self {
            Type::User { .. } => quote!(<#self as ::ribbit::Pack>::Packed),
            Type::Tight { .. } => quote!(#self),
        }
    }

    pub(crate) fn pack(&self, expression: TokenStream) -> TokenStream {
        match self {
            Type::User { .. } => quote!(#expression.pack()),
            Type::Tight { .. } => expression,
        }
    }

    pub(crate) fn unpack(&self, expression: TokenStream) -> TokenStream {
        match self {
            Type::User { .. } => quote!(#expression.unpack()),
            Type::Tight { .. } => expression,
        }
    }

    pub(crate) fn convert_to_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Type::Tight { tight, .. } => tight.convert_to_loose(expression),
            Type::User { .. } if self.is_generic() => {
                let loose = self.to_loose();
                quote! {
                    ::ribbit::convert::loose_to_loose::<_, #loose>(
                        ::ribbit::convert::packed_to_loose(#expression)
                    )
                }
            }
            Type::User { .. } => {
                quote!(::ribbit::convert::packed_to_loose(#expression))
            }
        }
    }

    pub(crate) fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            Type::Tight { tight, .. } => tight.convert_from_loose(expression),
            Type::User { .. } => {
                let packed = self.packed();
                quote!(unsafe { ::ribbit::convert::loose_to_packed::<#packed>(#expression) })
            }
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.as_tight().size()
    }

    pub(crate) fn is_non_zero(&self) -> bool {
        self.as_tight().is_non_zero()
    }

    pub(crate) fn is_zst(&self) -> bool {
        self.as_tight().size() == 0
    }

    pub(crate) fn mask(&self) -> u128 {
        self.as_tight().mask()
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Tight { tight } => tight.to_tokens(tokens),
            Self::User { path, .. } => path.to_tokens(tokens),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Tight { tight: left }, Type::Tight { tight: right }) => left == right,
            (Type::User { path: left, .. }, Type::User { path: right, .. }) => left == right,
            _ => false,
        }
    }
}
