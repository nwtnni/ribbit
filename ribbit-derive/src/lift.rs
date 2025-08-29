use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::ty::Loose;
use crate::ty::Tight;
use crate::ty::Type;
use crate::Or;

#[derive(Debug)]
pub(crate) enum Expr<'ir> {
    Constant(u128),
    Value {
        value: TokenStream,
        r#type: &'ir Type,
    },
    ValueSelf(&'ir Tight),

    /// Carve hole of size `type` at `offset` in `expr`.
    Hole {
        expr: Box<Self>,
        offset: u8,
        r#type: &'ir Type,
    },

    /// Extract `mask` bits at `offset` in `expr`.
    Extract {
        expr: Box<Self>,
        offset: u8,
        mask: Or<&'ir Type, &'ir ir::Discriminant>,
    },

    Combine {
        exprs: Vec<(u8, Self)>,
        tight: &'ir Tight,
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
        Self::ValueSelf(r#type.as_tight())
    }

    pub(crate) fn constant(value: u128) -> Self {
        Self::Constant(value)
    }

    pub(crate) fn or<I: IntoIterator<Item = (u8, Self)>>(tight: &'ir Tight, iter: I) -> Self {
        Self::Combine {
            exprs: iter.into_iter().collect(),
            tight,
        }
    }

    pub(crate) fn hole(self, offset: u8, r#type: &'ir Type) -> Self {
        Self::Hole {
            expr: Box::new(self),
            offset,
            r#type,
        }
    }

    pub(crate) fn extract(self, offset: u8, r#type: &'ir Type) -> Self {
        Self::Extract {
            expr: Box::new(self),
            offset,
            mask: Or::L(r#type),
        }
    }

    pub(crate) fn discriminant(self, discriminant: &'ir ir::Discriminant) -> Self {
        Self::Extract {
            expr: Box::new(self),
            offset: 0,
            mask: Or::R(discriminant),
        }
    }

    pub(crate) fn compile(self) -> TokenStream {
        let expr = self.optimize();
        let from = expr.type_intermediate().unwrap();
        let into = expr.type_final();
        expr.compile_intermediate().convert(from, into)
    }

    fn type_intermediate(&self) -> Result<TypeRef<'ir>, u128> {
        let r#type = match self {
            Expr::Constant(value) => return Err(*value),
            Expr::Value { r#type, .. } => (*r#type).into(),
            Expr::ValueSelf(tight) => (**tight).into(),

            Expr::Combine { tight, .. } => tight.to_loose().into(),

            Expr::Extract {
                mask: Or::L(r#type),
                ..
            } => r#type.as_tight().to_loose().into(),

            Expr::Extract {
                expr,
                mask: Or::R(_),
                ..
            }
            | Expr::Hole { expr, .. } => expr.type_intermediate().unwrap().to_loose().into(),
        };

        Ok(r#type)
    }

    fn type_final(&self) -> TypeRef<'ir> {
        match self {
            Expr::Constant { .. } => unreachable!(),
            Expr::ValueSelf(tight) | Expr::Combine { tight, .. } => (**tight).into(),

            Expr::Value { r#type, .. }
            | Expr::Extract {
                mask: Or::L(r#type),
                ..
            } => (*r#type).into(),

            // HACK: only used by discriminant matching,
            // where we don't want canonicalization to
            // convert back to tight type
            Expr::Extract {
                expr,
                mask: Or::R(_),
                ..
            } => expr.type_final().to_loose().into(),

            Expr::Hole { expr, .. } => expr.type_final(),
        }
    }

    fn compile_intermediate(&self) -> TokenStream {
        match self {
            Expr::Constant(value) => {
                proc_macro2::Literal::u128_unsuffixed(*value).to_token_stream()
            }

            Expr::Value { value, .. } => value.clone(),
            Expr::ValueSelf(_) => quote!(self.value),

            Expr::Extract { expr, offset, mask } => {
                let from = expr.type_intermediate().unwrap();
                let loose = from.to_loose();
                let expr = expr.compile_intermediate().convert(from, loose);
                let expr = Self::shift(expr, Dir::R, *offset, loose);

                match mask {
                    // Convert to extracted loose type
                    Or::L(r#type) => {
                        let into = r#type.to_loose();
                        let expr = expr.convert(loose, into);
                        let mask = into.literal(r#type.mask());
                        quote!((#expr & #mask))
                    }
                    Or::R(discriminant) => {
                        let mask = loose.literal(discriminant.mask);
                        quote!((#expr & #mask))
                    }
                }
            }

            Expr::Hole {
                expr,
                offset,
                r#type,
            } => {
                let from = expr.type_intermediate().unwrap();
                let loose = from.to_loose();

                let mask = loose.literal(!(r#type.mask() << *offset) & expr.type_final().mask());
                let expr = expr.compile_intermediate().convert(from, loose);

                quote!((#expr & #mask))
            }

            Expr::Combine { exprs, tight } if exprs.is_empty() => tight.to_loose().literal(0),

            Expr::Combine { exprs, tight } => {
                let into = tight.to_loose();

                let exprs = exprs.iter().map(|(offset, expr)| {
                    let expr = match expr.type_intermediate() {
                        Ok(from) => expr.compile_intermediate().convert(from, into),
                        Err(value) => return into.literal(value << offset).to_token_stream(),
                    };

                    Self::shift(expr, Dir::L, *offset, into)
                });

                quote!((#(#exprs )|*))
            }
        }
    }

    fn optimize(self) -> Self {
        match self {
            Self::Hole {
                expr,
                offset,
                r#type,
            } => {
                let expr = expr.optimize();
                if offset == 0 && expr.type_intermediate() == Ok(r#type.into()) {
                    Self::Constant(0)
                } else {
                    Self::Hole {
                        expr: Box::new(expr),
                        offset,
                        r#type,
                    }
                }
            }

            Self::Extract { expr, offset, mask } => {
                let expr = expr.optimize();

                if let Or::L(r#type) = &mask {
                    if offset == 0 && expr.type_intermediate() == Ok((*r#type).into()) {
                        return expr;
                    }
                }

                Self::Extract {
                    expr: Box::new(expr),
                    offset,
                    mask,
                }
            }

            Self::Combine { exprs, tight } => {
                let mut exprs = exprs
                    .into_iter()
                    .map(|(offset, expr)| (offset, expr.optimize()))
                    .filter(|(_, expr)| !matches!(expr, Self::Constant(0)))
                    .collect::<Vec<_>>();

                if exprs.len() == 1
                    && exprs[0].0 == 0
                    && exprs[0].1.type_intermediate() == Ok((*tight).into())
                {
                    exprs.remove(0).1
                } else {
                    Self::Combine { exprs, tight }
                }
            }

            _ => self,
        }
    }

    fn shift(expr: TokenStream, dir: Dir, shift: u8, loose: Loose) -> TokenStream {
        if shift == 0 {
            return expr;
        }

        let dir = match dir {
            Dir::L => quote!(<<),
            Dir::R => quote!(>>),
        };

        let shift = loose.literal(shift as u128);
        quote!((#expr #dir #shift))
    }
}

enum Dir {
    L,
    R,
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
enum TypeRef<'ir> {
    Type(Cow<'ir, Type>),
    Loose(Loose),
}

impl TypeRef<'_> {
    fn is_generic(&self) -> bool {
        match self {
            TypeRef::Type(r#type) => r#type.is_generic(),
            TypeRef::Loose(_) => false,
        }
    }

    fn to_loose(&self) -> Loose {
        match self {
            TypeRef::Type(r#type) => r#type.as_tight().to_loose(),
            TypeRef::Loose(loose) => *loose,
        }
    }

    fn mask(&self) -> u128 {
        match self {
            Self::Type(r#type) => r#type.as_tight().mask(),
            Self::Loose(loose) => loose.mask(),
        }
    }

    fn convert_to_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            TypeRef::Type(r#type) => r#type.convert_to_loose(expression),
            TypeRef::Loose(_) => expression,
        }
    }

    fn convert_from_loose(&self, expression: TokenStream) -> TokenStream {
        match self {
            TypeRef::Type(r#type) => r#type.convert_from_loose(expression),
            TypeRef::Loose(_) => expression,
        }
    }
}

impl ToTokens for TypeRef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypeRef::Type(r#type) => r#type.to_tokens(tokens),
            TypeRef::Loose(loose) => loose.to_tokens(tokens),
        }
    }
}

impl<'ir> From<Loose> for TypeRef<'ir> {
    fn from(loose: Loose) -> Self {
        Self::Loose(loose)
    }
}

impl<'ir> From<&'ir Type> for TypeRef<'ir> {
    fn from(r#type: &'ir Type) -> Self {
        Self::Type(Cow::Borrowed(r#type))
    }
}

impl<'ir> From<Tight> for TypeRef<'ir> {
    fn from(tight: Tight) -> Self {
        Self::Type(Cow::Owned(Type::Tight { path: None, tight }))
    }
}
