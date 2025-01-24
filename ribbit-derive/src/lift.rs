use core::ops::BitAnd;
use core::ops::BitOr;
use core::ops::Rem;
use core::ops::Shl;
use core::ops::Shr;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty;

pub(crate) trait Lift: Sized {
    fn lift(self) -> Value<Self>;
}

impl Lift for usize {
    fn lift(self) -> Value<Self> {
        Value::Compile(self)
    }
}

impl Lift for TokenStream {
    fn lift(self) -> Value<Self> {
        Value::Run(self)
    }
}

impl Lift for syn::Ident {
    fn lift(self) -> Value<Self> {
        Value::Run(self)
    }
}

pub(crate) trait Loosen: ToTokens {
    fn loose(&self) -> ty::Loose;
    fn is_zero(&self) -> bool;
}

impl Loosen for Box<dyn Loosen + '_> {
    fn loose(&self) -> ty::Loose {
        (**self).loose()
    }

    fn is_zero(&self) -> bool {
        (**self).is_zero()
    }
}

pub struct Loose<V> {
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
                ty::Tree::Leaf(_) => quote!(::ribbit::private::pack(#value)).to_tokens(tokens),
                ty::Tree::Node(node) => {
                    let loose = node.loosen();
                    let inner = quote!(::ribbit::private::convert::<_, #loose>(
                        ::ribbit::private::pack(#value)
                    ));
                    inner.to_tokens(tokens)
                }
            },
        }
    }
}

impl<V: ToTokens, T: Into<ty::Tree>> Rem<T> for Value<V> {
    type Output = Expression<'static, Loose<V>>;
    fn rem(self, tight: T) -> Self::Output {
        Expression {
            inner: Loose {
                value: self,
                ty: tight.into(),
            },
            op: Op::Pass,
        }
    }
}

impl<'a, V: Loosen> BitAnd<usize> for Expression<'a, V> {
    type Output = Expression<'static, Expression<'a, V>>;
    fn bitand(self, mask: usize) -> Self::Output {
        Expression {
            inner: self,
            op: Op::And(mask),
        }
    }
}

impl<'a, V: Loosen> Shl<usize> for Expression<'a, V> {
    type Output = Expression<'static, Expression<'a, V>>;
    fn shl(self, shift: usize) -> Self::Output {
        Expression {
            inner: self,
            op: Op::Shift { dir: Dir::L, shift },
        }
    }
}

impl<'a, V: Loosen> Shr<usize> for Expression<'a, V> {
    type Output = Expression<'static, Expression<'a, V>>;
    fn shr(self, shift: usize) -> Self::Output {
        Expression {
            inner: self,
            op: Op::Shift { dir: Dir::R, shift },
        }
    }
}

impl<'a, 'r, V: Loosen> BitOr<Box<dyn Loosen + 'r>> for Expression<'a, V> {
    type Output = Expression<'r, Expression<'a, V>>;
    fn bitor(self, rhs: Box<dyn Loosen + 'r>) -> Self::Output {
        Expression {
            inner: self,
            op: Op::Or(rhs),
        }
    }
}

impl<'a, V: Loosen> Rem<ty::Loose> for Expression<'a, V> {
    type Output = Expression<'static, Expression<'a, V>>;
    fn rem(self, loose: ty::Loose) -> Self::Output {
        Expression {
            inner: self,
            op: Op::Cast(loose),
        }
    }
}

impl<'a, V: Loosen> Rem<ty::Tree> for Expression<'a, V> {
    type Output = Tight<Expression<'a, V>>;
    fn rem(self, tight: ty::Tree) -> Self::Output {
        Tight {
            inner: self,
            ty: tight,
        }
    }
}

impl Rem<ty::Tree> for Box<dyn Loosen + '_> {
    type Output = Tight<Self>;
    fn rem(self, tight: ty::Tree) -> Self::Output {
        Tight {
            inner: self,
            ty: tight,
        }
    }
}

pub struct Expression<'ir, V> {
    inner: V,
    op: Op<'ir>,
}

pub(crate) enum Op<'ir> {
    Pass,
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

impl<V: Loosen> Loosen for Expression<'_, V> {
    fn loose(&self) -> ty::Loose {
        match &self.op {
            Op::Pass | Op::Shift { .. } | Op::And(_) | Op::Or(_) => self.inner.loose(),
            Op::Cast(loose) => *loose,
        }
    }

    fn is_zero(&self) -> bool {
        // Could be more precise, but this covers generated code
        match self.op {
            Op::Pass => self.inner.is_zero(),
            Op::And(0) => true,
            _ => false,
        }
    }
}

impl<V: Loosen> ToTokens for Expression<'_, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let inner = match &self.op {
            Op::Pass => inner,

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
                let cast = ty::Loose::cast(value.loose(), self.loose(), quote!(#value));
                quote!((#inner | #cast))
            }

            Op::Cast(loose) => ty::Loose::cast(self.inner.loose(), *loose, inner),
        };

        inner.to_tokens(tokens);
    }
}

pub struct Tight<V> {
    inner: V,
    ty: ty::Tree,
}

impl<V: Loosen> ToTokens for Tight<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let source = self.inner.loose();

        let inner = self.inner.to_token_stream();
        let inner = ty::Loose::cast(source, self.ty.loosen(), inner);

        match &self.ty {
            ty::Tree::Leaf(leaf) if leaf.is_loose() => inner,
            ty::Tree::Leaf(leaf) => quote!(unsafe {
                ::ribbit::private::unpack::<#leaf>(#inner)
            }),
            ty::Tree::Node(node) => quote! {
                unsafe { ::ribbit::private::unpack::<#node>(::ribbit::private::convert(#inner)) }
            },
        }
        .to_tokens(tokens)
    }
}
