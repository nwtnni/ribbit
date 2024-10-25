use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::repr;
use crate::repr::leaf;

trait Tree: ToTokens {
    fn ty(&self) -> repr::Tree;
}

pub(crate) trait Native: ToTokens {
    fn ty(&self) -> repr::Native;
}

pub(crate) trait NativeExt: Sized {
    fn apply(self, op: Op) -> Apply<Self> {
        Apply { inner: self, op }
    }

    fn into_repr(self, repr: repr::Tree) -> FromNative<Self> {
        FromNative {
            inner: self,
            target: repr,
        }
    }
}

impl<T: Native> NativeExt for T {}

impl<'a> Native for Box<dyn Native + 'a> {
    fn ty(&self) -> repr::Native {
        (**self).ty()
    }
}

pub(crate) fn zero(native: repr::Native) -> Zero {
    Zero(native)
}

pub(crate) fn lift<'ir, V>(inner: V, ty: impl Into<repr::Tree<'ir>>) -> Type<'ir, V> {
    Type {
        inner,
        ty: ty.into(),
    }
}

pub(crate) struct Type<'ir, V> {
    inner: V,
    ty: repr::Tree<'ir>,
}

impl<V> Type<'_, V> {
    pub(crate) fn into_native(self) -> IntoNative<Self> {
        IntoNative { inner: self }
    }
}

impl<V: ToTokens> Tree for Type<'_, V> {
    fn ty(&self) -> repr::Tree {
        self.ty
    }
}

impl<V: ToTokens> ToTokens for Type<'_, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens)
    }
}

pub(crate) struct IntoNative<V> {
    inner: V,
}

impl<V: Tree> Native for IntoNative<V> {
    fn ty(&self) -> repr::Native {
        self.inner.ty().as_native()
    }
}

impl<V: Tree> ToTokens for IntoNative<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let source = self.inner.ty();
        let inner = match source {
            repr::Tree::Node(_) => quote!(::ribbit::private::pack(#inner)),
            repr::Tree::Leaf(_) => inner,
        };

        let leaf = source.as_leaf();
        let inner = match (*leaf.nonzero, leaf.signed, *leaf.repr) {
            (_, true, _) | (true, _, leaf::Repr::Arbitrary(_)) => todo!(),
            (true, _, leaf::Repr::Native(_)) => quote!(#inner.get()),
            (false, _, leaf::Repr::Native(_)) => inner,
            (false, _, leaf::Repr::Arbitrary(_)) => quote!(#inner.value()),
        };

        inner.to_tokens(tokens)
    }
}

pub(crate) struct Apply<'ir, V> {
    inner: V,
    op: Op<'ir>,
}

pub(crate) enum Op<'ir> {
    Shift { dir: Dir, shift: usize },
    And(usize),
    Or(Box<dyn Native + 'ir>),
    Cast(repr::Native),
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

impl<V: Native> Native for Apply<'_, V> {
    fn ty(&self) -> repr::Native {
        match self.op {
            Op::Shift { .. } | Op::And(_) | Op::Or(_) => self.inner.ty(),
            Op::Cast(native) => native,
        }
    }
}

impl<V: Native> ToTokens for Apply<'_, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let inner = match &self.op {
            Op::Shift { dir: _, shift: 0 } => inner,
            Op::And(0) => self.ty().literal(0).to_token_stream(),
            Op::Shift { dir, shift } => {
                let shift = self.ty().literal(*shift);
                quote!((#inner #dir #shift))
            }
            Op::And(value) => {
                let value = self.ty().literal(*value);
                quote!((#inner & #value))
            }
            Op::Or(value) => {
                let native = self.ty();
                match value.ty() == native {
                    false => quote!((#inner | (#value as #native))),
                    true => quote!((#inner | #value)),
                }
            }
            Op::Cast(native) if *native == self.inner.ty() => inner,
            Op::Cast(native) => quote!((#inner as #native)),
        };

        inner.to_tokens(tokens);
    }
}

pub(crate) struct FromNative<'ir, V> {
    inner: V,
    target: repr::Tree<'ir>,
}

impl<'ir, V: Native> ToTokens for FromNative<'ir, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();
        let source = self.inner.ty();

        let native = self.target.as_native();
        let inner = match source == native {
            false => quote!((#inner as #native)),
            true => inner,
        };

        let leaf = self.target.as_leaf();
        let inner = match (*leaf.nonzero, leaf.signed, *leaf.repr) {
            (_, true, _) | (true, _, leaf::Repr::Arbitrary(_)) => todo!(),
            (true, _, leaf::Repr::Native(_)) => quote!(match #leaf::new(#inner) {
                None => panic!(),
                Some(output) => output,
            }),
            (false, _, leaf::Repr::Native(native)) if native == source => inner,
            (false, _, leaf::Repr::Native(native)) => quote!((#inner as #native)),
            (false, _, leaf::Repr::Arbitrary(arbitrary)) => {
                let mask = Literal::usize_unsuffixed(arbitrary.mask());
                quote!(#leaf::new(#inner & #mask))
            }
        };

        let inner = match self.target {
            repr::Tree::Leaf(_) => inner,
            repr::Tree::Node(node) => {
                quote!(unsafe { ::ribbit::private::unpack::<#node>(#inner) })
            }
        };

        inner.to_tokens(tokens);
    }
}

pub(crate) struct Zero(repr::Native);

impl Native for Zero {
    fn ty(&self) -> repr::Native {
        self.0
    }
}

impl ToTokens for Zero {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.literal(0).to_tokens(tokens)
    }
}
