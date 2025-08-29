use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::ty::Loose;
use crate::ty::Type;
use crate::Or;

#[derive(Debug)]
pub(crate) enum Expr<'ir> {
    Constant {
        value: u128,
    },
    Value {
        value: TokenStream,
        r#type: &'ir Type,
    },

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
        r#type: &'ir Type,
    },
}

impl<'ir> Expr<'ir> {
    pub(crate) fn new(value: impl ToTokens, r#type: &'ir Type) -> Self {
        Self::Value {
            value: value.to_token_stream(),
            r#type,
        }
    }

    pub(crate) fn constant(value: u128) -> Self {
        Self::Constant { value }
    }

    pub(crate) fn or<I: IntoIterator<Item = (u8, Self)>>(r#type: &'ir Type, iter: I) -> Self {
        Self::Combine {
            exprs: iter.into_iter().collect(),
            r#type,
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
        let from = self.type_intermediate().unwrap();
        let into = self.type_final();
        self.compile_intermediate().convert(from, into)
    }

    fn type_intermediate(&self) -> Result<TypeRef<'ir>, u128> {
        match self {
            Expr::Constant { value } => Err(*value),
            Expr::Value { r#type, .. } => Ok(TypeRef::Type(r#type)),

            Expr::Combine { r#type, .. }
            | Expr::Extract {
                mask: Or::L(r#type),
                ..
            } => Ok(TypeRef::Loose(*r#type.as_tight().loosen())),

            Expr::Extract {
                expr,
                mask: Or::R(_),
                ..
            }
            | Expr::Hole { expr, .. } => {
                Ok(TypeRef::Loose(expr.type_intermediate().unwrap().to_loose()))
            }
        }
    }

    fn type_final(&self) -> TypeRef<'ir> {
        match self {
            Expr::Constant { .. } => unreachable!(),
            Expr::Value { r#type, .. }
            | Expr::Combine { r#type, .. }
            | Expr::Extract {
                mask: Or::L(r#type),
                ..
            } => TypeRef::Type(r#type),

            // HACK: only used by discriminant matching,
            // where we don't want canonicalization to
            // convert back to tight type
            Expr::Extract {
                expr,
                mask: Or::R(_),
                ..
            } => TypeRef::Loose(expr.type_final().to_loose()),

            Expr::Hole { expr, .. } => expr.type_final(),
        }
    }

    fn compile_intermediate(&self) -> TokenStream {
        match self {
            Expr::Constant { .. } => {
                unreachable!("[INTERNAL ERROR]: constant with no type")
            }

            Expr::Value { value, .. } => value.clone(),

            Expr::Extract { expr, offset, mask } => {
                let from = expr.type_intermediate().unwrap();
                let loose = from.to_loose();
                let expr = expr.compile_intermediate().convert(from, loose);

                let shift = loose.literal(*offset as u128);
                let expr = quote!((#expr >> #shift));

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

            Expr::Combine { exprs, r#type } if exprs.is_empty() => r#type.to_loose().literal(0),

            Expr::Combine { exprs, r#type } => {
                let into = r#type.to_loose();

                let exprs = exprs.iter().map(|(offset, expr)| {
                    let expr = match expr.type_intermediate() {
                        Ok(from) => expr.compile_intermediate().convert(from, into),
                        Err(value) => into.literal(value).to_token_stream(),
                    };

                    let offset = into.literal(*offset as u128);
                    quote!((#expr << #offset))
                });

                quote!((#(#exprs )|*))
            }
        }
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

            match (from, into) {
                (TypeRef::Loose(_), TypeRef::Loose(into)) => {
                    quote!(::ribbit::convert::loose_to_loose::<_, #into>(#value))
                }

                (from @ TypeRef::Loose(_), into) => {
                    let value = convert_impl(value, from, TypeRef::Loose(into.to_loose()));

                    quote!(
                        unsafe { ::ribbit::convert::loose_to_packed(#value) }
                    )
                }

                (from, into) => convert_impl(
                    quote!(
                        ::ribbit::convert::packed_to_loose(#value)
                    ),
                    TypeRef::Loose(from.to_loose()),
                    into,
                ),
            }
        }

        convert_impl(self, from.into(), into.into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TypeRef<'ir> {
    Type(&'ir Type),
    Loose(Loose),
}

impl TypeRef<'_> {
    fn to_loose(self) -> Loose {
        match self {
            TypeRef::Type(r#type) => *r#type.as_tight().loosen(),
            TypeRef::Loose(loose) => loose,
        }
    }

    pub(crate) fn mask(&self) -> u128 {
        match self {
            Self::Type(r#type) => r#type.as_tight().mask(),
            Self::Loose(loose) => loose.mask(),
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
        Self::Type(r#type)
    }
}
