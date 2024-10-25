use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ty;
use crate::ty::leaf;

trait Tree: ToTokens {
    fn ty(&self) -> &ty::Tree;
}

pub(crate) trait Native: ToTokens {
    fn ty(&self) -> ty::Native;
}

pub(crate) trait NativeExt: Sized {
    fn apply(self, op: Op) -> Apply<Self> {
        Apply { inner: self, op }
    }

    fn convert_to_ty(self, ty: impl Into<ty::Tree>) -> ConvertFromNative<Self> {
        ConvertFromNative {
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
    pub(crate) fn convert_to_native(self) -> ConvertToNative<Self> {
        ConvertToNative { inner: self }
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

pub(crate) struct ConvertToNative<V> {
    inner: V,
}

impl<V: Tree> Native for ConvertToNative<V> {
    fn ty(&self) -> ty::Native {
        self.inner.ty().to_native()
    }
}

impl<V: Tree> ToTokens for ConvertToNative<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();

        let source = self.inner.ty();
        let inner = match source {
            ty::Tree::Node(_) => quote!(::ribbit::private::pack(#inner)),
            ty::Tree::Leaf(_) => inner,
        };

        let leaf = source.to_leaf();
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

pub(crate) struct ConvertFromNative<V> {
    inner: V,
    target: ty::Tree,
}

impl<V: Native> ToTokens for ConvertFromNative<V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = self.inner.to_token_stream();
        let source = self.inner.ty();

        let native = self.target.to_native();
        let inner = match source == native {
            false => quote!((#inner as #native)),
            true => inner,
        };

        let leaf = self.target.to_leaf();
        let inner = match (*leaf.nonzero, leaf.signed, *leaf.repr) {
            (_, true, _) | (true, _, leaf::Repr::Arbitrary(_)) => todo!(),
            (true, _, leaf::Repr::Native(_)) => quote!(unsafe { #leaf::new_unchecked(#inner) }),
            (false, _, leaf::Repr::Native(native)) if native == source => inner,
            (false, _, leaf::Repr::Native(native)) => quote!((#inner as #native)),
            (false, _, leaf::Repr::Arbitrary(arbitrary)) => {
                let mask = native.literal(arbitrary.mask());
                quote!(unsafe { #leaf::new_unchecked(#inner & #mask) })
            }
        };

        let inner = match &self.target {
            ty::Tree::Leaf(_) => inner,
            ty::Tree::Node(node) => {
                quote!(unsafe { ::ribbit::private::unpack::<#node>(#inner) })
            }
        };

        inner.to_tokens(tokens);
    }
}

pub(crate) struct Zero(ty::Native);

impl Native for Zero {
    fn ty(&self) -> ty::Native {
        self.0
    }
}

impl ToTokens for Zero {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.literal(0).to_tokens(tokens)
    }
}
