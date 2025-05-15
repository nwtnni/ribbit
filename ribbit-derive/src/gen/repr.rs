use proc_macro2::TokenStream;
use quote::quote;

use crate::ir;

pub(crate) fn repr(
    ir @ ir::Ir {
        tight: repr,
        item,
        data,
        ..
    }: &ir::Ir,
) -> TokenStream {
    let size = repr.size();
    let generics = ir.generics_bounded(None);
    let (generics_impl, generics_ty, generics_where) = generics.split_for_impl();

    // https://github.com/MrGVSV/to_phantom/blob/main/src/lib.rs
    let lifetimes = generics.lifetimes().map(|lifetime| quote!(&#lifetime ()));
    let tys = generics.type_params();
    let vis = &item.vis;
    let ident = &item.ident;

    let r#struct = quote! {
        #vis struct #ident #generics_ty {
            value: #repr,
            r#type: ::ribbit::private::PhantomData<fn(#(#lifetimes),*) -> (#(#tys),*)>,
        }
    };

    let unpack = match data {
        ir::Data::Struct(r#struct) => {
            let fields = r#struct.iter().map(|field| {
                let name = field.ident.unescaped("");
                let get = field.ident.escaped();

                quote!(#name: self.#get())
            });

            quote! {
                Self::Unpack {
                    #(
                        #fields
                    ),*
                }
            }
        }
        ir::Data::Enum(_) => quote!(self.unpack()),
    };

    let attrs = &item.attrs;
    let unpacked = ir::Enum::unpacked(&item.ident);

    quote! {
        #(#attrs)*
        #r#struct

        unsafe impl #generics_impl ::ribbit::Pack for #ident #generics_ty #generics_where {
            const BITS: usize = #size;
            type Unpack = #unpacked #generics_ty;
            type Tight = #repr;
            type Loose = <#repr as ::ribbit::Pack>::Loose;
            fn unpack(&self) -> Self::Unpack {
                #unpack
            }
        }
    }
}
