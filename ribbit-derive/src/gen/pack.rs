use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct StructOpt {}

impl StructOpt {}

pub(crate) fn pack(ir: &ir::Ir) -> TokenStream {
    let unpacked = ir.ident_unpacked();
    let packed = ir.ident_packed();

    let pack = match &ir.data {
        ir::Data::Struct(r#struct) => {
            let arguments = r#struct
                .iter_nonzero()
                .map(|ir::Field { ident, ty, .. }| ty.pack(quote!(self.#ident)));
            quote!(#packed::new(#(#arguments),*))
        }
        ir::Data::Enum(_) => {
            quote!(#packed::new(self))
        }
    };

    let tight = &ir.tight;
    let size = tight.size();

    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    quote! {
        unsafe impl #generics_impl ::ribbit::Pack for #unpacked #generics_ty #generics_where {
            const BITS: usize = #size;
            type Packed = #packed #generics_ty;
            type Tight = #tight;
            type Loose = <#tight as ::ribbit::Pack>::Loose;

            #[inline]
            fn pack(self) -> #packed #generics_ty {
                #pack
            }
        }
    }
}
