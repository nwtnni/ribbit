use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

use crate::ir;

// NOTE: does not use `ir::CommonOpt` because `vis` needs special handling and `skip` is ignored
#[derive(FromMeta, Clone, Debug, Default)]
pub(crate) struct ItemOpt {
    vis: Option<syn::Visibility>,
    rename: Option<syn::Ident>,
}

impl ItemOpt {
    pub(crate) fn name<'ir>(&'ir self, unpacked: &'ir syn::Ident) -> Cow<'ir, syn::Ident> {
        self.rename
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(format_ident!("{}Packed", unpacked)))
    }

    pub(crate) fn vis<'ir>(&'ir self, default: &'ir syn::Visibility) -> &'ir syn::Visibility {
        self.vis.as_ref().unwrap_or(default)
    }
}

pub(crate) fn packed(item: &ir::Item) -> TokenStream {
    let opt = &item.opt().packed;
    let forward = &item.opt().forward;
    let vis = opt
        .vis
        .clone()
        .map(ir::raise_vis)
        .unwrap_or_else(|| item.vis.clone());
    let packed = item.ident_packed();
    let tight = item.tight();

    let generics = item.generics();
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
