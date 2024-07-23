use bitvec::bitbox;
use darling::ast::Style;
use darling::util::SpannedValue;
use proc_macro2::Literal;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::error::bail;
use crate::input;

pub(crate) fn new<'input>(
    attr: &'input input::Attr,
    input: &'input syn::DeriveInput,
    item: &'input input::Item,
) -> darling::Result<Struct<'input>> {
    match &item.data {
        darling::ast::Data::Enum(_) => todo!(),
        darling::ast::Data::Struct(r#struct) => {
            match r#struct.style {
                Style::Unit | Style::Tuple => todo!(),
                Style::Struct => (),
            }

            let mut bits = bitbox![0; *attr.size];
            let mut fields = Vec::new();

            for field in &r#struct.fields {
                let uninit = FieldUninit::new(field);
                let size = uninit.size();
                let offset = match uninit.offset() {
                    Offset::Implicit => match bits.first_zero() {
                        Some(offset) => offset,
                        None => bail!(field => crate::Error::Overflow {
                            offset: 0,
                            available: 0,
                            required: size
                        }),
                    },
                };

                let prefix = &mut bits[offset..];
                match prefix.first_one().unwrap_or_else(|| prefix.len()) {
                    len if len < size => bail!(field => crate::Error::Overflow {
                        offset,
                        available: len,
                        required: size
                    }),
                    _ => prefix[..size].fill(true),
                }

                fields.push(uninit.with_offset(offset));
            }

            if bits.not_all() {
                bail!(input => crate::Error::Underflow {
                    bits,
                })
            }

            Ok(Struct {
                repr: StructRepr::new(&attr.size),
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            })
        }
    }
}

pub(crate) struct Struct<'input> {
    repr: StructRepr,
    attrs: &'input [syn::Attribute],
    vis: &'input syn::Visibility,
    ident: &'input syn::Ident,
    fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    pub(crate) fn repr(&self) -> &StructRepr {
        &self.repr
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub(crate) fn fields(&self) -> &[Field] {
        &self.fields
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = &self.repr;
        let ident = self.ident;
        let vis = self.vis;
        let attrs = self.attrs;

        quote! {
            #( #attrs )*
            #vis struct #ident {
                value: #repr,
            }
        }
        .to_tokens(tokens)
    }
}

pub(crate) type Field<'input> = FieldInner<'input, usize>;
pub(crate) type FieldUninit<'input> = FieldInner<'input, Offset>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Offset {
    Implicit,
}

pub(crate) struct FieldInner<'input, O> {
    vis: &'input syn::Visibility,
    ident: Option<&'input syn::Ident>,
    repr: FieldRepr,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(field: &'input input::Field) -> Self {
        Self {
            vis: &field.vis,
            ident: field.ident.as_ref(),
            repr: FieldRepr::new(&field.ty),
            offset: Offset::Implicit,
        }
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            ident: self.ident,
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
    pub(crate) fn size(&self) -> usize {
        self.repr.size()
    }

    pub(crate) fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    pub(crate) fn repr(&self) -> &FieldRepr {
        &self.repr
    }

    pub(crate) fn ident(&self) -> Option<&syn::Ident> {
        self.ident
    }
}

pub(crate) struct FieldRepr {
    span: Span,
    ty: Tree,
}

impl FieldRepr {
    fn new(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Array(_) => todo!(),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Path(path) => Self::from_path(path),
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(_) => todo!(),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }

    fn from_path(type_path @ syn::TypePath { qself, path }: &syn::TypePath) -> Self {
        if qself.is_some() {
            todo!();
        }

        if path.leading_colon.is_some() {
            todo!()
        }

        if path.segments.len() > 1 {
            todo!();
        }

        let segment = path.segments.first().unwrap();

        if !segment.arguments.is_none() {
            todo!();
        }

        let ident = segment.ident.to_string();

        if !ident.is_ascii() {
            todo!();
        }

        let signed = match &ident[0..1] {
            "u" => false,
            "i" => true,
            _ => todo!(),
        };

        let size = ident[1..].parse::<usize>().unwrap();

        if let Some(native) = match (signed, size) {
            (false, 8) => Some(Native::U8),
            (false, 16) => Some(Native::U16),
            (false, 32) => Some(Native::U32),
            (false, 64) => Some(Native::U64),
            _ => None,
        } {
            return FieldRepr {
                span: type_path.span(),
                ty: Tree::Leaf(Leaf::Native(native)),
            };
        }

        FieldRepr {
            span: type_path.span(),
            ty: Tree::Leaf(Leaf::Arbitrary(Arbitrary { size })),
        }
    }

    fn size(&self) -> usize {
        self.ty.size()
    }

    pub(crate) fn ty(&self) -> &Tree {
        &self.ty
    }
}

impl ToTokens for FieldRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        quote_spanned!(self.span=> #ty).to_tokens(tokens)
    }
}

pub(crate) enum Tree {
    Leaf(Leaf),
}

impl Tree {
    pub(crate) fn as_native(&self) -> Native {
        match self {
            Tree::Leaf(leaf) => leaf.as_native(),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Tree::Leaf(leaf) => leaf.size(),
        }
    }
}

impl ToTokens for Tree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Tree::Leaf(leaf) => leaf.to_tokens(tokens),
        }
    }
}

pub(crate) struct StructRepr {
    span: Span,
    ty: Leaf,
}

impl StructRepr {
    fn new(size: &SpannedValue<usize>) -> Self {
        Self {
            span: size.span(),
            ty: Leaf::new(**size),
        }
    }

    pub(crate) fn ty(&self) -> &Leaf {
        &self.ty
    }
}

impl ToTokens for StructRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        quote_spanned!(self.span=> #ty).to_tokens(tokens)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Leaf {
    Native(Native),
    Arbitrary(Arbitrary),
}

impl Leaf {
    fn new(size: usize) -> Self {
        assert!(size <= 64);
        match size {
            8 => Leaf::Native(Native::U8),
            16 => Leaf::Native(Native::U16),
            32 => Leaf::Native(Native::U32),
            64 => Leaf::Native(Native::U64),
            _ => Leaf::Arbitrary(Arbitrary { size }),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Leaf::Native(native) => native.size(),
            Leaf::Arbitrary(arbitrary) => arbitrary.size(),
        }
    }

    pub(crate) fn as_native(&self) -> Native {
        match *self {
            Leaf::Native(native) => native,
            Leaf::Arbitrary(arbitrary) => arbitrary.as_native(),
        }
    }
}

impl ToTokens for Leaf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Leaf::Native(native) => native.to_tokens(tokens),
            Leaf::Arbitrary(arbitrary) => arbitrary.to_tokens(tokens),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Arbitrary {
    size: usize,
}

impl Arbitrary {
    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn as_native(&self) -> Native {
        match self.size {
            0..=7 => Native::U8,
            9..=15 => Native::U16,
            17..=31 => Native::U32,
            33..=63 => Native::U64,
            _ => unreachable!(),
        }
    }
}

impl ToTokens for Arbitrary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = format_ident!("u{}", self.size);
        quote!(::ribbit::private::#repr).to_tokens(tokens);
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Native {
    U8,
    U16,
    U32,
    U64,
}

impl Native {
    fn size(&self) -> usize {
        match self {
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 => 32,
            Self::U64 => 64,
        }
    }
}

impl ToTokens for Native {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let repr = match self {
            Native::U8 => quote!(u8),
            Native::U16 => quote!(u16),
            Native::U32 => quote!(u32),
            Native::U64 => quote!(u64),
        };

        quote!(::ribbit::private::#repr).to_tokens(tokens)
    }
}

pub(crate) fn mask(size: usize) -> usize {
    (1 << size) - 1
}

pub(crate) fn literal(native: Native, value: usize) -> Literal {
    match native {
        Native::U8 => Literal::u8_suffixed(value.try_into().unwrap()),
        Native::U16 => Literal::u16_suffixed(value.try_into().unwrap()),
        Native::U32 => Literal::u32_suffixed(value.try_into().unwrap()),
        Native::U64 => Literal::u64_suffixed(value.try_into().unwrap()),
    }
}
