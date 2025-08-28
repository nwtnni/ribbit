use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty;
use crate::Or;

#[derive(Debug)]
pub(crate) enum Expr<'ir> {
    Constant {
        value: u128,
    },
    Value {
        value: TokenStream,
        r#type: Ty<'ir>,
    },
    Hole {
        expr: Box<Self>,
        offset: u8,
        r#type: Ty<'ir>,
    },
    Extract {
        expr: Box<Self>,
        offset: u8,
        r#type: Or<Ty<'ir>, u128>,
    },
    Combine {
        exprs: Vec<(u8, Self)>,
        r#type: Ty<'ir>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ty<'ir> {
    Node(&'ir ty::Node),
    Leaf(&'ir ty::Tight),
}

impl<'ir> From<&'ir ty::Node> for Ty<'ir> {
    fn from(node: &'ir ty::Node) -> Self {
        Self::Node(node)
    }
}

impl<'ir> From<&'ir ty::Tight> for Ty<'ir> {
    fn from(leaf: &'ir ty::Tight) -> Self {
        Self::Leaf(leaf)
    }
}

impl<'ir> From<&'ir ty::Tree> for Ty<'ir> {
    fn from(tree: &'ir ty::Tree) -> Self {
        match tree {
            ty::Tree::Node(node) => Self::Node(node),
            ty::Tree::Leaf(leaf) => Self::Leaf(leaf),
        }
    }
}

impl<'ir> Ty<'ir> {
    fn convert(from: Self, into: Self, value: impl ToTokens) -> TokenStream {
        if from == into {
            return value.to_token_stream();
        }

        match (from, into) {
            (Ty::Node(_), Ty::Node(_)) => todo!(),

            (Ty::Leaf(from), Ty::Leaf(into)) if from.is_loose() && into.is_loose() => {
                quote!(::ribbit::convert::loose_to_loose::<_, #into>(#value))
            }

            (Ty::Leaf(from), into) if from.is_loose() => {
                let value = Self::convert(from.into(), into.loosen().tighten().into(), value);

                quote!(
                    unsafe { ::ribbit::convert::loose_to_packed::<#into>(#value) }
                )
            }

            (from, Ty::Leaf(into)) if into.is_loose() => Self::convert(
                from.loosen().tighten().into(),
                into.into(),
                quote!(
                    ::ribbit::convert::packed_to_loose::<#from>(#value)
                ),
            ),

            _ => todo!("conversion {:?} to {:?}", from, into),
        }
    }

    fn mask(&self) -> u128 {
        match self {
            Self::Node(node) => node.tighten().mask(),
            Self::Leaf(leaf) => leaf.mask(),
        }
    }

    fn loosen(&self) -> &ty::Loose {
        match self {
            Self::Node(node) => node.loosen(),
            Self::Leaf(leaf) => leaf.loosen(),
        }
    }
}

impl ToTokens for Ty<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Node(node) => node.to_tokens(tokens),
            Self::Leaf(leaf) => leaf.to_tokens(tokens),
        }
    }
}

impl<'ir> Expr<'ir> {
    pub(crate) fn new(value: impl ToTokens, r#type: impl Into<Ty<'ir>>) -> Self {
        Self::Value {
            value: value.to_token_stream(),
            r#type: r#type.into(),
        }
    }

    pub(crate) fn constant(value: u128) -> Self {
        Self::Constant { value }
    }

    pub(crate) fn or<I: IntoIterator<Item = (u8, Self)>>(
        r#type: impl Into<Ty<'ir>>,
        iter: I,
    ) -> Self {
        Self::Combine {
            exprs: iter.into_iter().collect(),
            r#type: r#type.into(),
        }
    }

    pub(crate) fn hole(self, offset: u8, r#type: impl Into<Ty<'ir>>) -> Self {
        Self::Hole {
            expr: Box::new(self),
            offset,
            r#type: r#type.into(),
        }
    }

    pub(crate) fn extract(self, offset: u8, r#type: impl Into<Ty<'ir>>) -> Self {
        Self::Extract {
            expr: Box::new(self),
            offset,
            r#type: Or::L(r#type.into()),
        }
    }

    pub(crate) fn mask(self, offset: u8, mask: u128) -> Self {
        Self::Extract {
            expr: Box::new(self),
            offset,
            r#type: Or::R(mask),
        }
    }

    pub(crate) fn canonicalize(self) -> impl ToTokens + 'ir {
        Canonical(self)
    }

    fn loose(&self) -> Result<&ty::Loose, u128> {
        match self {
            Expr::Constant { value } => Err(*value),
            Expr::Value { r#type, .. }
            | Expr::Combine { r#type, .. }
            | Expr::Extract {
                r#type: Or::L(r#type),
                ..
            } => Ok(r#type.loosen()),
            Expr::Extract {
                expr,
                r#type: Or::R(_),
                ..
            }
            | Expr::Hole { expr, .. } => Ok(expr.loose()?),
        }
    }

    fn tight(&self) -> Ty<'ir> {
        match self {
            Expr::Constant { .. } => unreachable!(),
            Expr::Value { r#type, .. }
            | Expr::Combine { r#type, .. }
            | Expr::Extract {
                r#type: Or::L(r#type),
                ..
            } => *r#type,
            // HACK: only used by discriminant matching,
            // where we don't want canonicalization to
            // convert back to tight type
            Expr::Extract {
                expr,
                r#type: Or::R(_),
                ..
            } => expr.loose().unwrap().tighten().into(),
            Expr::Hole { expr, .. } => expr.tight(),
        }
    }

    fn compile(&self) -> TokenStream {
        match self {
            Expr::Constant { .. } => {
                unreachable!("[INTERNAL ERROR]: constant with no type")
            }

            Expr::Value { value, r#type } => {
                Ty::convert(*r#type, r#type.loosen().tighten().into(), value)
            }

            Expr::Extract {
                expr,
                offset,
                r#type,
            } => {
                let from = expr.loose().expect("[INTERNAL ERROR]: constant in shift");

                let shift = from.literal(*offset as u128);

                let expr = expr.compile();
                let expr = quote!((#expr >> #shift));

                // Convert to extracted loose type
                match r#type {
                    Or::L(r#type) => {
                        let into = r#type.loosen();
                        let expr = Ty::convert(from.tighten().into(), into.tighten().into(), expr);
                        let mask = into.literal(r#type.mask());
                        quote!((#expr & #mask))
                    }
                    Or::R(mask) => {
                        let mask = from.literal(*mask);
                        quote!((#expr & #mask))
                    }
                }
            }

            Expr::Hole {
                expr,
                offset,
                r#type,
            } => {
                let mask = expr
                    .loose()
                    .expect("[INTERNAL ERROR]: constant in shift")
                    .literal(!(r#type.mask() << *offset) & expr.tight().mask());

                let expr = expr.compile();
                quote!((#expr & #mask))
            }

            Expr::Combine { exprs, r#type } if exprs.is_empty() => r#type.loosen().literal(0),

            Expr::Combine { exprs, r#type } => {
                let into = r#type.loosen();

                let exprs = exprs.iter().map(|(offset, expr)| {
                    let expr = match expr.loose() {
                        Ok(from) => Ty::convert(
                            from.tighten().into(),
                            into.tighten().into(),
                            expr.compile(),
                        ),
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

struct Canonical<'ir>(Expr<'ir>);

impl<'ir> ToTokens for Canonical<'ir> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = self.0.loose().unwrap();
        let into = self.0.tight();
        Ty::convert(from.tighten().into(), into, self.0.compile()).to_tokens(tokens);
    }
}
