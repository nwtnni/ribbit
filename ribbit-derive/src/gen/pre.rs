use core::ops::Deref as _;

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
                .flat_map(|variant| extract_assertions(&variant.r#struct)),
        ),
    };

    quote! {
        #[doc(hidden)]
        const _RIBBIT_PRECONDITION: () = {
            #(#assertions ;)*
        };
    }
}

pub(crate) fn precondition() -> TokenStream {
    quote! {
        let _: () = Self::_RIBBIT_PRECONDITION;
    }
}

fn extract_assertions<'ir>(r#struct: &'ir ir::Struct) -> impl Iterator<Item = TokenStream> + 'ir {
    let fields = r#struct
        .fields
        .iter()
        .map(|field| &field.r#type)
        // Only need to check user-defined types
        .filter(|r#type| r#type.is_user());

    let nonzero = fields
        .clone()
        .filter(|r#type| r#type.is_nonzero())
        .map(|r#type| {
            let span = r#type.span();
            let r#type = r#type.deref();
            quote_spanned! {span=>
                ::ribbit::private::assert_nonzero::<#r#type>();
            }
        });

    let pack = fields.map(|r#type| {
        let assert = match r#type.is_generic() {
            true => quote!(assert_size_ge),
            false => quote!(assert_size_eq),
        };

        let span = r#type.span();
        let size = r#type.size();
        let r#type = r#type.deref();
        quote_spanned! {span=>
            ::ribbit::private::#assert::<#r#type>(#size);
        }
    });

    nonzero.chain(pack)
}
