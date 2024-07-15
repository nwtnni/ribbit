use bitvec::bitbox;
use darling::ast::Style;
use darling::util::SpannedValue;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

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
                size: &attr.size,
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            })
        }
    }
}

pub(crate) struct Struct<'input> {
    size: &'input SpannedValue<usize>,
    attrs: &'input [syn::Attribute],
    vis: &'input syn::Visibility,
    ident: &'input syn::Ident,
    fields: Vec<Field<'input>>,
}

impl Struct<'_> {
    pub(crate) fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub(crate) fn fields(&self) -> &[Field] {
        &self.fields
    }
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident;
        let vis = self.vis;
        let attrs = self.attrs;

        let repr = format_ident!("u{}", **self.size, span = self.size.span());
        let repr = match **self.size {
            8 | 16 | 32 | 64 => quote!(#repr),
            _ => quote!(::ribbit::private::arbitrary_int::#repr),
        };

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
    ty: Type<'input>,
    offset: O,
}

impl<'input> FieldUninit<'input> {
    fn new(field: &'input input::Field) -> Self {
        Self {
            vis: &field.vis,
            ident: field.ident.as_ref(),
            ty: Type::new(&field.ty),
            offset: Offset::Implicit,
        }
    }

    fn with_offset(self, offset: usize) -> Field<'input> {
        Field {
            vis: self.vis,
            ident: self.ident,
            ty: self.ty,
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
        self.ty.size()
    }

    pub(crate) fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    pub(crate) fn ty(&self) -> &Type {
        &self.ty
    }

    pub(crate) fn ident(&self) -> Option<&syn::Ident> {
        self.ident
    }
}

pub(crate) enum Type<'input> {
    Builtin {
        path: &'input syn::TypePath,
        builtin: Builtin,
    },
    Arbitrary {
        path: &'input syn::TypePath,
        size: usize,
    },
}

impl<'input> Type<'input> {
    fn new(ty: &'input syn::Type) -> Self {
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

    fn from_path(type_path @ syn::TypePath { qself, path }: &'input syn::TypePath) -> Self {
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

        if let Some(builtin) = match (signed, size) {
            (false, 8) => Some(Builtin::U8),
            (false, 16) => Some(Builtin::U16),
            (false, 32) => Some(Builtin::U32),
            (false, 64) => Some(Builtin::U64),
            _ => None,
        } {
            return Type::Builtin {
                path: type_path,
                builtin,
            };
        }

        Type::Arbitrary {
            path: type_path,
            size,
        }
    }

    fn size(&self) -> usize {
        match self {
            Type::Builtin { path: _, builtin } => builtin.size(),
            Type::Arbitrary { path: _, size } => *size,
        }
    }
}

impl ToTokens for Type<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Type::Builtin { path, builtin: _ } => path,
            Type::Arbitrary { path, size: _ } => path,
        }
        .to_tokens(tokens)
    }
}

pub(crate) enum Builtin {
    U8,
    U16,
    U32,
    U64,
}

impl Builtin {
    fn size(&self) -> usize {
        match self {
            Builtin::U8 => 8,
            Builtin::U16 => 16,
            Builtin::U32 => 32,
            Builtin::U64 => 64,
        }
    }
}

impl AsRef<str> for Builtin {
    fn as_ref(&self) -> &str {
        match self {
            Builtin::U8 => "u8",
            Builtin::U16 => "u16",
            Builtin::U32 => "u32",
            Builtin::U64 => "u64",
        }
    }
}
