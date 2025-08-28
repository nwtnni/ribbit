use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::ir;
use crate::lift::Lift as _;
use crate::Or;

pub(crate) fn get<'ir>(ir: &'ir ir::Ir) -> impl Iterator<Item = TokenStream> + 'ir {
    let ty_struct = ir.tight();

    match &ir.data {
        ir::Data::Struct(r#struct) => Or::L({
            let newtype = r#struct.is_newtype();
            r#struct.iter().map(move |field| {
                let value = get_field(newtype, 0, ty_struct, field);
                let vis = field.vis;
                let get = field.ident.escaped();
                let ty = field.ty.packed();

                quote! {
                    #[inline]
                    #vis const fn #get(&self) -> #ty {
                        let _: () = Self::_RIBBIT_ASSERT_LAYOUT;
                        #value
                    }
                }
            })
        }),
        ir::Data::Enum(_) => Or::R(core::iter::empty()),
    }
}

pub(crate) fn get_field(
    newtype: bool,
    shift: usize,
    ty_struct: &crate::ty::Tight,
    field: &ir::Field,
) -> TokenStream {
    let ty_field = &*field.ty;

    let value = quote!(self.value);

    match newtype {
        // Forward underlying type directly
        true if ty_field.is_leaf() && shift == 0 => value.to_token_stream(),
        // Skip conversion through loose types
        true if shift == 0 => {
            (value.lift() % ty_struct.clone() % ty_field.clone()).to_token_stream()
        }
        _ => {
            ((((value.lift() % ty_struct.clone()) >> (shift + field.offset)) % ty_field.loosen())
                & ty_field.mask())
                % ty_field.clone()
        }
        .to_token_stream(),
    }
}
