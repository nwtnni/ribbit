use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty;

trait Tree: ToTokens {
    fn ty(&self) -> &ty::Tree;
}

pub(crate) trait Native: ToTokens {
    fn ty(&self) -> ty::Native;
    fn is_zero(&self) -> bool;
}

pub(crate) trait NativeExt: Sized {
    fn apply(self, op: Op) -> Apply<Self> {
        Apply { inner: self, op }
    }

    fn native_to_ty(self, ty: impl Into<ty::Tree>) -> NativeToTy<Self> {
        NativeToTy {
            inner: self,
            target: ty.into(),
        }
    }
}

impl<T: Native> NativeExt for T {}

impl<'a> Native for Box<dyn Native + 'a> {
    fn ty(&self) -> ty::Native {
        (**self).ty()
    }

    fn is_zero(&self) -> bool {
        (**self).is_zero()
    }
}

pub(crate) fn zero(native: ty::Native) -> Zero {
    Zero(native)
}

pub(crate) fn lift<V>(inner: V, ty: impl Into<ty::Tree>) -> Type<V> {
    Type {
        inner,
        ty: ty.into(),
    }
}

pub(crate) struct Type<V> {
    inner: V,
    ty: ty::Tree,
}

impl<V> Type<V> {
    pub(crate) fn ty_to_native(self) -> TyToNative<Self> {
        TyToNative { inner: self }
    }
}

impl<V: ToTokens> Tree for Type<V> {
    fn ty(&self) -> &ty::Tree {
        &self.ty
    }
}

impl<V: ToTokens> ToTokens for Type<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens)
    }
}

pub(crate) struct TyToNative<V> {
    inner: V,
}

impl<V: Tree> Native for TyToNative<V> {
    fn ty(&self) -> ty::Native {
        self.inner.ty().to_native()
    }

    fn is_zero(&self) -> bool {
        false
    }
}

impl<V: Tree> ToTokens for TyToNative<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        match self.inner.ty() {
            ty::Tree::Leaf(leaf) if leaf.is_native() => inner,
            ty::Tree::Leaf(_) | ty::Tree::Node(_) => {
                quote!(::ribbit::private::pack(#inner))
            }
        }
        .to_tokens(tokens)
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
    Cast(ty::Native),
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
    fn ty(&self) -> ty::Native {
        match self.op {
            Op::Shift { .. } | Op::And(_) | Op::Or(_) => self.inner.ty(),
            Op::Cast(native) => native,
        }
    }

    fn is_zero(&self) -> bool {
        // Could be more precise, but this covers generated code
        matches!(self.op, Op::And(0))
    }
}

impl<V: Native> ToTokens for Apply<'_, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let inner = match &self.op {
            Op::Shift { dir: _, shift: 0 } => inner,
            Op::Shift { dir, shift } => {
                let shift = self.ty().literal(*shift);
                quote!((#inner #dir #shift))
            }

            Op::And(0) => self.ty().literal(0).to_token_stream(),
            Op::And(mask) if *mask == self.ty().mask() => inner,
            Op::And(value) => {
                let value = self.ty().literal(*value);
                quote!((#inner & #value))
            }

            Op::Or(value) if self.inner.is_zero() => value.to_token_stream(),
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

pub(crate) struct NativeToTy<V> {
    inner: V,
    target: ty::Tree,
}

impl<V: Native> ToTokens for NativeToTy<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();
        let source = self.inner.ty();

        let target = &self.target;
        let native = self.target.to_native();

        // Convert source type to target native type
        let inner = match source == native {
            false => quote!((#inner as #native)),
            true => inner,
        };

        let inner = match *target == ty::Tree::from(ty::Leaf::from(native)) {
            true => inner,
            false => quote!(unsafe { ::ribbit::private::unpack::<#target>(#inner) }),
        };

        inner.to_tokens(tokens)
    }
}

pub(crate) struct Zero(ty::Native);

impl Native for Zero {
    fn ty(&self) -> ty::Native {
        self.0
    }

    fn is_zero(&self) -> bool {
        true
    }
}

impl ToTokens for Zero {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.literal(0).to_tokens(tokens)
    }
}
