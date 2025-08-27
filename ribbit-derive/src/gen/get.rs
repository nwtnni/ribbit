use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift::Lift as _;
use crate::ty;
use crate::Or;

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = ir.tight();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L({
            let newtype = r#struct.is_newtype();
            r#struct.iter().map(move |field| {
                let ty_field = &*field.ty;

                let value = quote!(self.value);

                let value_field = match newtype {
                    // Forward underlying type directly
                    true if ty_field.is_leaf() => value.to_token_stream(),
                    // Skip conversion through loose types
                    true => (value.lift() % ty_struct.clone() % ty_field.clone()).to_token_stream(),
                    #[allow(clippy::precedence)]
                    false => {
                        ((value.lift() % ty_struct.clone() >> field.offset) % ty_field.loosen()
                            & ty_field.mask())
                            % ty_field.clone()
                    }
                    .to_token_stream(),
                };

                let vis = field.vis;
                let get = field.ident.escaped();

                let ty_field = match ty_field {
                    ty::Tree::Node(node) => quote!(<#node as ::ribbit::Pack>::Packed),
                    ty::Tree::Leaf(leaf) => leaf.to_token_stream(),
                };

                quote! {
                    #[inline]
                    #vis const fn #get(&self) -> #ty_field {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        #value_field
                    }
                }
            })
        }),
        ir::Data::Enum(_) => Or::R(core::iter::empty()),
    }
}
