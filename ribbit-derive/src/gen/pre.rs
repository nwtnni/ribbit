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
        .filter(|r#type| r#type.is_user());

    let nonzero = fields
        .clone()
        .filter(|r#type| r#type.is_nonzero())
        .map(|r#type| {
            let packed = r#type.packed();
            quote! {
                ::ribbit::private::assert_impl_all!(#packed: ::ribbit::NonZero)
            }
        });

    let pack = fields.map(|r#type| {
        let expected = r#type.size_expected();
        let actual = r#type.size_actual();
        let generic = r#type.is_generic();

        let (message, compare) = match generic {
            true => (quote!("fit"), quote!(>=)),
            false => (quote!("match"), quote!(==)),
        };

        quote_spanned! {r#type.span()=>
            ::ribbit::private::concat_assert!(
                #expected #compare #actual,
                "Annotated size ",
                #expected,
                " of type ",
                stringify!(#r#type),
                " does not ",
                #message,
                " actual size ",
                #actual,
            )
        }
    });

    nonzero.chain(pack)
}
