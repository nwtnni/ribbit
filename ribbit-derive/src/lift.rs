use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty;

pub(crate) trait Loosen: ToTokens {
    fn loose(&self) -> ty::Loose;
    fn is_zero(&self) -> bool;
}

pub(crate) trait LoosenExt: Sized {
    fn apply(self, op: Op) -> Apply<Self> {
        Apply { inner: self, op }
    }

    fn tighten(self, ty: impl Into<ty::Tree>) -> Tight<Self> {
        Tight {
            inner: self,
            target: ty.into(),
        }
    }
}

impl<T: Loosen> LoosenExt for T {}

impl<'a> Loosen for Box<dyn Loosen + 'a> {
    fn loose(&self) -> ty::Loose {
        (**self).loose()
    }

    fn is_zero(&self) -> bool {
        (**self).is_zero()
    }
}

pub(crate) fn constant(value: usize, native: ty::Loose) -> Loose<TokenStream> {
    Loose {
        ty: native.into(),
        value: Value::Compile(value),
    }
}

pub(crate) fn lift<V>(value: V, ty: impl Into<ty::Tree>) -> Loose<V> {
    Loose {
        ty: ty.into(),
        value: Value::Run(value),
    }
}

pub(crate) struct Loose<V> {
    ty: ty::Tree,
    value: Value<V>,
}

pub(crate) enum Value<V> {
    Compile(usize),
    Run(V),
}

impl<V: ToTokens> Loosen for Loose<V> {
    fn loose(&self) -> ty::Loose {
        self.ty.loosen()
    }

    fn is_zero(&self) -> bool {
        match &self.value {
            Value::Compile(0) => true,
            Value::Compile(_) => false,
            Value::Run(_) => false,
        }
    }
}

impl<V: ToTokens> ToTokens for Loose<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.value {
            Value::Compile(value) => self.ty.loosen().literal(*value).to_tokens(tokens),
            Value::Run(value) => match &self.ty {
                ty::Tree::Leaf(leaf) if leaf.is_loose() => value.to_tokens(tokens),
                ty::Tree::Leaf(_) | ty::Tree::Node(_) => {
                    quote!(::ribbit::private::pack(#value)).to_tokens(tokens)
                }
            },
        }
    }
}

pub(crate) struct Apply<'ir, V> {
    inner: V,
    op: Op<'ir>,
}

pub(crate) enum Op<'ir> {
    Shift { dir: Dir, shift: usize },
    And(usize),
    Or(Box<dyn Loosen + 'ir>),
    Cast(ty::Loose),
}

#[derive(Copy, Clone)]
pub(crate) enum Dir {
    L,
    R,
}

impl ToTokens for Dir {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Dir::L => quote!(<<),
            Dir::R => quote!(>>),
        }
        .to_tokens(tokens)
    }
}

impl<V: Loosen> Loosen for Apply<'_, V> {
    fn loose(&self) -> ty::Loose {
        match &self.op {
            Op::Shift { .. } | Op::And(_) | Op::Or(_) => self.inner.loose(),
            Op::Cast(native) => *native,
        }
    }

    fn is_zero(&self) -> bool {
        // Could be more precise, but this covers generated code
        matches!(self.op, Op::And(0))
    }
}

impl<V: Loosen> ToTokens for Apply<'_, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let inner = match &self.op {
            Op::Shift { dir: _, shift: 0 } => inner,
            Op::Shift { dir, shift } => {
                let shift = self.loose().literal(*shift);
                quote!((#inner #dir #shift))
            }

            Op::And(0) => self.loose().literal(0).to_token_stream(),
            Op::And(mask) if *mask == self.loose().mask() => inner,
            Op::And(value) => {
                let value = self.loose().literal(*value);
                quote!((#inner & #value))
            }

            Op::Or(value) if self.inner.is_zero() => value.to_token_stream(),
            Op::Or(value) if value.is_zero() => self.inner.to_token_stream(),
            Op::Or(value) => {
                let native = self.loose();
                match value.loose() == native {
                    false => quote!((#inner | (#value as #native))),
                    true => quote!((#inner | #value)),
                }
            }

            Op::Cast(native) if *native == self.inner.loose() => inner,
            Op::Cast(native) => quote!((#inner as #native)),
        };

        inner.to_tokens(tokens);
    }
}

pub(crate) struct Tight<V> {
    inner: V,
    target: ty::Tree,
}

impl<V: Loosen> ToTokens for Tight<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();
        let source = self.inner.loose();

        let target = &self.target;
        let native = self.target.loosen();

        // Convert source type to target native type
        let inner = match source == native {
            false => quote!((#inner as #native)),
            true => inner,
        };

        let inner = match *target == ty::Tree::from(ty::Tight::from(native)) {
            true => inner,
            false => quote!(unsafe { ::ribbit::private::unpack::<#target>(#inner) }),
        };

        inner.to_tokens(tokens)
    }
}
