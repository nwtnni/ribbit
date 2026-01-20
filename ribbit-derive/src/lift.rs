use core::ops::Deref;
use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::r#type::Loose;
use crate::r#type::Tight;
use crate::Type;

#[derive(Debug)]
pub(crate) enum Expr<'ir> {
    Constant(u128),
    Value {
        value: TokenStream,
        r#type: &'ir Type,
    },
    ValueTight {
        value: TokenStream,
        tight: &'ir Tight,
    },

    And {
        expr: Box<Self>,
        mask: u128,
    },

    Or(Box<[Self]>),

    Shift {
        expr: Box<Self>,
        #[expect(private_interfaces)]
        dir: Dir,
        by: u8,
    },
}

impl<'ir> Expr<'ir> {
    pub(crate) fn value(value: impl ToTokens, r#type: &'ir Type) -> Self {
        Self::Value {
            value: value.to_token_stream(),
            r#type,
        }
    }

    pub(crate) fn value_self(r#type: &'ir Type) -> Self {
        Self::ValueTight {
            value: quote!(self.value),
            tight: r#type.as_tight(),
        }
    }

    pub(crate) fn value_tight(value: TokenStream, tight: &'ir Tight) -> Self {
        Self::ValueTight { value, tight }
    }

    pub(crate) fn constant(value: u128) -> Self {
        Self::Constant(value)
    }

    pub(crate) fn and(self, mask: u128) -> Self {
        Self::And {
            expr: Box::new(self),
            mask,
        }
    }

    pub(crate) fn or<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self::Or(iter.into_iter().collect())
    }

    pub(crate) fn shift_left(self, by: u8) -> Self {
        Self::Shift {
            expr: Box::new(self),
            dir: Dir::L,
            by,
        }
    }

    pub(crate) fn shift_right(self, by: u8) -> Self {
        Self::Shift {
            expr: Box::new(self),
            dir: Dir::R,
            by,
        }
    }

    #[expect(private_bounds)]
    pub(crate) fn compile(self, r#type: impl Into<TypeRef<'ir>>) -> TokenStream {
        let r#type = r#type.into();

        Canonical {
            r#type,
            expr: self.optimize(),
        }
        .to_token_stream()
    }

    fn unify(&self, loose: Loose) -> Loose {
        match self {
            Expr::Constant(_) => loose,
            Expr::ValueTight { tight, .. } => tight.to_loose().max(loose),
            Expr::Value { r#type, .. } => r#type.to_loose().max(loose),
            Expr::And { expr, .. } | Expr::Shift { expr, .. } => expr.unify(loose),
            Expr::Or(exprs) => exprs
                .iter()
                .map(|expr| expr.unify(loose))
                .max()
                .unwrap_or(Loose::N8),
        }
    }

    fn compile_intermediate(&self, loose: Loose) -> TokenStream {
        match self {
            Self::Constant(value) => loose.literal(*value),

            Self::Value { value, r#type } => value.clone().convert(*r#type, loose),
            Self::ValueTight { value, tight } => value.clone().convert(*tight, loose),

            Self::And { expr, mask } => {
                let expr = expr.compile_intermediate(loose);
                let mask = loose.literal(*mask);
                quote!((#expr & #mask))
            }

            // NOTE: must compile to a value of type loose, not unit `()`.
            Self::Or(exprs) if exprs.is_empty() => loose.literal(0),
            Self::Or(exprs) => {
                let exprs = exprs
                    .into_iter()
                    .map(|expr| expr.compile_intermediate(loose));

                quote!(( #(#exprs)|* ))
            }
            Self::Shift { expr, dir, by } => {
                let expr = expr.compile_intermediate(loose);
                let by = loose.literal(*by as u128);
                quote!((#expr #dir #by))
            }
        }
    }

    fn optimize(self) -> Self {
        match self {
            Self::And { expr: _, mask: 0 } => Self::Constant(0),

            Self::And { expr, mask } => match expr.optimize() {
                Self::Constant(value) => Self::Constant(value & mask),
                expr => Self::And {
                    expr: Box::new(expr),
                    mask,
                },
            },

            Self::Or(exprs) => {
                let mut constants = 0;

                let mut exprs = exprs
                    .into_vec()
                    .into_iter()
                    .map(|expr| expr.optimize())
                    .filter(|expr| match expr {
                        Self::Constant(value) => {
                            constants |= value;
                            false
                        }
                        _ => true,
                    })
                    .collect::<Vec<_>>();

                if constants != 0 {
                    exprs.push(Self::Constant(constants));
                }

                match exprs.len() {
                    0 => Self::Constant(0),
                    1 => exprs.remove(0),
                    _ => Self::Or(exprs.into_boxed_slice()),
                }
            }

            Self::Shift {
                expr,
                dir: _,
                by: 0,
            } => *expr,

            Self::Shift { expr, dir, by } => match expr.optimize() {
                Self::Constant(value) => Self::Constant(match dir {
                    Dir::L => value << by,
                    Dir::R => value >> by,
                }),
                expr => Self::Shift {
                    expr: Box::new(expr),
                    dir,
                    by,
                },
            },

            _ => self,
        }
    }
}

struct Canonical<'ir> {
    r#type: TypeRef<'ir>,
    expr: Expr<'ir>,
}

impl<'ir> ToTokens for Canonical<'ir> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let r#final = &self.r#type;
        let loose = self.expr.unify(r#final.to_loose());

        // Short circuit if no conversion through loose is necessary
        match &self.expr {
            Expr::Value { value, r#type } if TypeRef::from(*r#type) == *r#final => {
                return value.to_tokens(tokens)
            }
            Expr::ValueTight { value, tight } if TypeRef::from(*tight) == *r#final => {
                return value.to_tokens(tokens)
            }
            _ => (),
        }

        self.expr
            .compile_intermediate(loose)
            .convert(loose, r#final.clone())
            .to_tokens(tokens)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Dir {
    L,
    R,
}

impl ToTokens for Dir {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Dir::L => quote!(<<),
            Dir::R => quote!(>>),
        }
        .to_tokens(tokens);
    }
}

trait Convert {
    fn convert<'ir>(
        self,
        from: impl Into<TypeRef<'ir>>,
        into: impl Into<TypeRef<'ir>>,
    ) -> TokenStream;
}

impl Convert for TokenStream {
    fn convert<'ir>(
        self,
        from: impl Into<TypeRef<'ir>>,
        into: impl Into<TypeRef<'ir>>,
    ) -> TokenStream {
        fn convert_impl<'ir>(
            value: TokenStream,
            from: TypeRef<'ir>,
            into: TypeRef<'ir>,
        ) -> TokenStream {
            if from == into {
                return value;
            }

            let from_loose = from.to_loose();
            let into_loose = into.to_loose();

            let value = from.convert_to_loose(value);

            let value = if !from.is_generic() && !into.is_generic() {
                match from_loose == into_loose {
                    true => value,
                    false => quote!((#value as #into_loose)),
                }
            } else {
                let from_loose = match from.is_generic() {
                    true => quote!(_),
                    false => quote!(#from_loose),
                };

                let into_loose = match into.is_generic() {
                    true => quote!(_),
                    false => quote!(#into_loose),
                };

                quote!(::ribbit::convert::loose_to_loose::<#from_loose, #into_loose>(#value))
            };

            into.convert_from_loose(value)
        }

        convert_impl(self, from.into(), into.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypeRef<'ir>(Cow<'ir, Type>);

impl<'ir> Deref for TypeRef<'ir> {
    type Target = Type;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'ir> From<Loose> for TypeRef<'ir> {
    fn from(loose: Loose) -> Self {
        Self::from(loose.as_tight())
    }
}

impl<'ir> From<&'ir Type> for TypeRef<'ir> {
    fn from(r#type: &'ir Type) -> Self {
        Self(Cow::Borrowed(r#type))
    }
}

impl<'ir> From<&'ir Tight> for TypeRef<'ir> {
    fn from(tight: &'ir Tight) -> Self {
        Self(Cow::Owned(Type::Tight {
            path: None,
            tight: *tight,
        }))
    }
}
