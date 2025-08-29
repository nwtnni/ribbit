use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;

use crate::ir;
use crate::Or;

pub(crate) fn pre(ir: &ir::Ir) -> TokenStream {
    let assertions = match &ir.data {
        ir::Data::Struct(r#struct) => Or::L(extract_assertions(r#struct)),
        ir::Data::Enum(r#enum) => Or::R(
            r#enum
                .variants
                .iter()
                .filter(|variant| !variant.extract)
                .flat_map(|variant| extract_assertions(&variant.r#struct)),
        ),
    };

    quote! {
        #[doc(hidden)]
        const _RIBBIT_ASSERT_LAYOUT: () = {
            #(#assertions ;)*
        };
    }
}

fn extract_assertions<'ir>(r#struct: &'ir ir::Struct) -> impl Iterator<Item = TokenStream> + 'ir {
    let fields = r#struct
        .fields
        .iter()
        .map(|field| &field.ty)
        // Only need to check user-defined types
        .filter(|ty| ty.is_user());

    let nonzero = fields
        .clone()
        .filter(|ty| ty.is_nonzero())
        .map(|ty| quote!(::ribbit::private::assert_impl_all!(#ty: ::ribbit::NonZero)));

    let pack = fields.map(|ty| {
        let expected = ty.size_expected();
        let actual = ty.size_actual();
        quote_spanned! {ty.span()=>
            ::ribbit::private::concat_assert!(
                #expected >= #actual,
                "Annotated size ",
                #expected,
                " is too small to fit type ",
                stringify!(#ty),
                " of size ",
                #actual,
            )
        }
    });

    nonzero.chain(pack)
}
