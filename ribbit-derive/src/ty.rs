mod arbitrary;
mod loose;
pub(crate) mod tight;

pub(crate) use arbitrary::Arbitrary;
use darling::usage::CollectTypeParams as _;
use darling::usage::IdentSet;
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
use crate::Spanned;

#[derive(Clone, Debug, Eq)]
pub(crate) enum Type {
    Tight {
        path: Option<TypePath>,
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
        opt_struct: &ir::StructOpt,
        opt_field: &ir::FieldOpt,
        ty_params: &IdentSet,
        ty: syn::Type,
    ) -> darling::Result<Spanned<Self>> {
        let syn::Type::Path(path) = ty else {
            bail!(ty=> Error::UnsupportedType)
        };

        let span = path.span();

        if let Some(tight) = Tight::from_path(&path) {
            if let Some(expected) = opt_field.size.filter(|size| **size != tight.size()) {
                bail!(span=> Error::WrongSize {
                    expected: *expected,
                    actual: tight.size(),
                    ty: tight,
                });
            }

            return Ok(Spanned::new(
                Self::Tight {
                    path: Some(path),
                    tight,
                },
                span,
            ));
        };

        // For convenience, forward nonzero and size annotations
        // for newtype structs.
        let nonzero = match (newtype, opt_field.nonzero) {
            (false, nonzero) | (true, nonzero @ Some(_)) => nonzero,
            (true, None) => opt_struct.nonzero,
        };
        let size = match (newtype, opt_field.size) {
            (false, size) | (true, size @ Some(_)) => size,
            (true, None) => opt_struct.size,
        };

        let Some(size) = size else {
            bail!(span=> Error::OpaqueSize);
        };

        let tight = Tight::from_size(nonzero.as_deref().copied().unwrap_or(false), *size);

        let tight = match tight {
            Ok(tight) => tight,
            Err(error) => bail!(span=> error),
        };

        let uses = std::iter::once(&path)
            .collect_type_params_cloned(&darling::usage::Purpose::Declare.into(), ty_params);

        Ok(Spanned::new(Self::User { path, uses, tight }, span))
    }

    pub(crate) fn is_user(&self) -> bool {
        matches!(self, Self::User { .. })
    }

    pub(crate) fn is_generic(&self) -> bool {
        matches!(self, Self::User { uses, .. } if !uses.is_empty())
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
            Type::Tight {
                tight: Tight::Unit, ..
            } => quote!(()),
            Type::Tight {
                tight: Tight::Bool, ..
            } => {
                let zero = proc_macro2::Literal::usize_unsuffixed(0);
                quote!((#expression != #zero))
            }
            Type::Tight {
                tight: Tight::Loose { signed: false, .. },
                ..
            } => expression,
            Type::Tight {
                tight:
                    Tight::Loose {
                        signed: true,
                        loose,
                    },
                ..
            } => quote!((#expression as #loose)),

            Type::User { .. } if self.is_generic() => {
                let loose = self.to_loose();
                let packed = self.packed();
                quote!(unsafe {
                    ::ribbit::convert::loose_to_packed::<#packed>(
                        ::ribbit::convert::loose_to_loose::<#loose, _>(
                            #expression
                        )
                    )
                })
            }

            // Skip validation logic in `NonZero` and `Arbitrary` constructors
            Type::Tight {
                tight: Tight::NonZero(_) | Tight::Arbitrary(_),
                ..
            }
            | Type::User { .. } => {
                let packed = self.packed();
                quote!(unsafe { ::ribbit::convert::loose_to_packed::<#packed>(#expression) })
            }
        }
    }

    pub(crate) fn size_actual(&self) -> TokenStream {
        let packed = self.packed();
        quote!(<#packed as ::ribbit::Unpack>::BITS)
    }

    pub(crate) fn size_expected(&self) -> usize {
        self.as_tight().size()
    }

    pub(crate) fn is_nonzero(&self) -> bool {
        self.as_tight().is_nonzero()
    }

    pub(crate) fn mask(&self) -> u128 {
        self.as_tight().mask()
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Tight {
                path: Some(path), ..
            } => path.to_tokens(tokens),
            Self::Tight { path: None, tight } => tight.to_tokens(tokens),
            Self::User { path, .. } => path.to_tokens(tokens),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Type::Tight {
                    path: _,
                    tight: left,
                },
                Type::Tight {
                    path: _,
                    tight: right,
                },
            ) => left == right,
            (
                Type::User {
                    path: left_path,
                    uses: _,
                    tight: left_tight,
                },
                Type::User {
                    path: right_path,
                    uses: _,
                    tight: right_tight,
                },
            ) => left_tight == right_tight && left_path == right_path,
            _ => false,
        }
    }
}
