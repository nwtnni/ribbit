use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct ItemOpt(ir::CommonOpt);

impl ItemOpt {
    pub(crate) fn name<'ir>(&'ir self, unpacked: &'ir syn::Ident) -> Cow<'ir, syn::Ident> {
        self.0
            .rename_with(|| Cow::Owned(format_ident!("{}Packed", unpacked)))
    }

    pub(crate) fn vis<'ir>(&'ir self, default: &'ir syn::Visibility) -> &'ir syn::Visibility {
        self.0.vis(default)
    }
}

pub(crate) fn packed(ir: &ir::Ir) -> TokenStream {
    let opt = &ir.opt().packed;
    let forward = &ir.opt().forward;
    let vis = opt.0.vis(&ir.vis);
    let packed = ir.ident_packed();
    let tight = ir.r#type().as_tight();

    let generics = ir.generics();
    let (generics_impl, generics_type, generics_where) = generics.split_for_impl();

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes();
    let types = generics.type_params();

    quote! {
        #forward
        #[repr(transparent)]
        #vis struct #packed #generics_type {
            value: #tight,
            r#type: ::ribbit::PhantomData<fn(#(&#lifetimes ()),*) -> (#(#types),*)>,
        }

        #[automatically_derived]
        impl #generics_impl Copy for #packed #generics_type #generics_where {}

        #[automatically_derived]
        impl #generics_impl Clone for #packed #generics_type #generics_where {
            fn clone(&self) -> Self {
                *self
            }
        }
    }
}
