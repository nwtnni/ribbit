use std::borrow::Cow;

use bitvec::bitbox;
use quote::format_ident;

use crate::error::bail;
use crate::input;
use crate::repr::leaf;
use crate::repr::Leaf;
use crate::repr::Tree;
use crate::Spanned;

pub(crate) fn new<'input>(
    attr: &'input input::Attr,
    input: &'input syn::DeriveInput,
    item: &'input input::Item,
) -> darling::Result<Struct<'input>> {
    match &item.data {
        darling::ast::Data::Enum(_) => todo!(),
        darling::ast::Data::Struct(r#struct) => {
            let mut bits = bitbox![0; *attr.size];
            let mut fields = Vec::new();

            for (index, field) in r#struct.fields.iter().enumerate() {
                let uninit = FieldUninit::new(index, field)?;
                let size = *uninit.size();
                let offset = match uninit.offset() {
                    Offset::Explicit(offset) => match *offset >= bits.len() {
                        false => offset,
                        true => bail!(field => crate::Error::Overflow {
                            offset: *offset,
                            available: 0,
                            required: size,
                        }),
                    },
                    Offset::Implicit => match bits.first_zero() {
                        Some(offset) => Spanned::new(offset, field.span()),
                        None => bail!(field => crate::Error::Overflow {
                            offset: 0,
                            available: 0,
                            required: size
                        }),
                    },
                };

                let prefix = &mut bits[*offset..];
                match prefix.first_one().unwrap_or_else(|| prefix.len()) {
                    len if len < size => bail!(offset=> crate::Error::Overflow {
                        offset: *offset,
                        available: len,
                        required: size
                    }),
                    _ => prefix[..size].fill(true),
                }

                fields.push(uninit.with_offset(*offset));
            }

            if bits.not_all() {
                bail!(input => crate::Error::Underflow {
                    bits,
                })
            }

            let leaf = Leaf::new(
                attr.nonzero
                    .map(Spanned::from)
                    .unwrap_or_else(|| false.into()),
                attr.size.into(),
            );

            if let (true, leaf::Repr::Arbitrary(_)) = (*leaf.nonzero, *leaf.repr) {
                bail!(leaf.nonzero=> crate::Error::ArbitraryNonZero);
            }

            Ok(Struct {
                repr: leaf.into(),
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            })
        }
    }
}

pub(crate) struct Struct<'input> {
    pub(crate) repr: Spanned<Leaf>,
    pub(crate) attrs: &'input [syn::Attribute],
    pub(crate) vis: &'input syn::Visibility,
    pub(crate) ident: &'input syn::Ident,
    pub(crate) fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    pub(crate) fn repr(&self) -> &Spanned<Leaf> {
        &self.repr
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub(crate) fn fields(&self) -> &[Field] {
        &self.fields
    }
}

pub(crate) type Field<'input> = FieldInner<'input, usize>;
pub(crate) type FieldUninit<'input> = FieldInner<'input, Offset>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Offset {
    Implicit,
    Explicit(Spanned<usize>),
}

pub(crate) struct FieldInner<'input, O> {
    vis: &'input syn::Visibility,
    name: FieldName<'input>,
    repr: Spanned<Tree<'input>>,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(index: usize, field: &'input input::Field) -> darling::Result<Self> {
        Ok(Self {
            vis: &field.vis,
            name: FieldName::new(index, field.ident.as_ref()),
            repr: Tree::from_ty(
                &field.ty,
                field.nonzero.map(Spanned::from),
                field.size.map(Spanned::from),
            )?,
            offset: match field.offset {
                None => Offset::Implicit,
                Some(offset) => Offset::Explicit(offset.into()),
            },
        })
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            name: self.name,
            repr: self.repr,
            offset,
        }
    }
}

impl<'input, O: Copy> FieldInner<'input, O> {
    pub(crate) fn offset(&self) -> O {
        self.offset
    }
}

impl<'input, O: Copy> FieldInner<'input, O> {
    pub(crate) fn size(&self) -> Spanned<usize> {
        match &*self.repr {
            Tree::Node(node) => node.size(),
            Tree::Leaf(leaf) => leaf.size(),
        }
    }

    pub(crate) fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    pub(crate) fn repr(&self) -> &Spanned<Tree> {
        &self.repr
    }

    pub(crate) fn name(&self) -> &FieldName {
        &self.name
    }

    pub(crate) fn nonzero(&self) -> Spanned<bool> {
        self.repr.nonzero()
    }
}

pub(crate) enum FieldName<'input> {
    Named(&'input syn::Ident),
    Unnamed(usize),
}

impl<'input> FieldName<'input> {
    fn new(index: usize, name: Option<&'input syn::Ident>) -> Self {
        name.map(FieldName::Named)
            .unwrap_or_else(|| FieldName::Unnamed(index))
    }

    pub(crate) fn unescaped(&self, prefix: &'static str) -> syn::Ident {
        match self {
            FieldName::Named(named) => format_ident!("{}_{}", prefix, named),
            FieldName::Unnamed(unnamed) => format_ident!("{}_{}", prefix, unnamed),
        }
    }

    pub(crate) fn escaped(&self) -> Cow<syn::Ident> {
        match self {
            FieldName::Named(named) => Cow::Borrowed(*named),
            FieldName::Unnamed(unnamed) => Cow::Owned(format_ident!("_{}", unnamed)),
        }
    }
}
