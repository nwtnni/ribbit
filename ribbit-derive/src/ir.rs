use bitvec::bitbox;
use darling::ast::Style;
use darling::util::SpannedValue;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;

use crate::input;

pub(crate) fn new<'input>(
    attr: &'input input::Attr,
    input: &'input syn::DeriveInput,
    item: &'input input::Item,
) -> Struct<'input> {
    match &item.data {
        darling::ast::Data::Enum(_) => todo!(),
        darling::ast::Data::Struct(r#struct) => {
            match r#struct.style {
                Style::Unit | Style::Tuple => todo!(),
                Style::Struct => (),
            }

            let mut storage = bitbox![0; *attr.size];
            let fields = r#struct
                .fields
                .iter()
                .map(|field| {
                    let offset = storage.first_zero().unwrap();
                    let field = Field::new(field, offset);
                    storage[offset..][..field.size()].fill(true);
                    field
                })
                .collect();

            assert!(storage.all());

            Struct {
                size: &attr.size,
                attrs: &input.attrs,
                vis: &input.vis,
                ident: &input.ident,
                fields,
            }
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

pub(crate) struct Field<'input> {
    vis: &'input syn::Visibility,
    ident: Option<&'input syn::Ident>,
    ty: Type<'input>,
    offset: usize,
}

impl<'input> Field<'input> {
    fn new(field: &'input input::Field, offset: usize) -> Self {
        Field {
            vis: &field.vis,
            ident: field.ident.as_ref(),
            ty: Type::new(&field.ty),
            offset,
        }
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

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
        ident: &'input syn::Ident,
        builtin: Builtin,
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

    fn from_path(syn::TypePath { qself, path }: &'input syn::TypePath) -> Self {
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

        if let Some(builtin) = [Builtin::U8, Builtin::U16, Builtin::U32, Builtin::U64]
            .into_iter()
            .find(|builtin| segment.ident == builtin)
        {
            Type::Builtin {
                ident: &segment.ident,
                builtin,
            }
        } else {
            todo!()
        }
    }

    fn size(&self) -> usize {
        match self {
            Type::Builtin { ident: _, builtin } => builtin.size(),
        }
    }
}

impl ToTokens for Type<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Type::Builtin { ident, builtin: _ } => ident.to_tokens(tokens),
        }
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
