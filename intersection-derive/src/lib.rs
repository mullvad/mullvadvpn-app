extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

// FIXME: rename to Intersection
#[proc_macro_derive(IntersectionDerive)]
pub fn derive_answer_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => derive_for_struct(&input, data),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
    .unwrap_or_else(|e| e.into_compile_error())
    .into()
}

fn derive_for_struct(input: &DeriveInput, data: &syn::DataStruct) -> syn::Result<TokenStream> {
    let my_type = &input.ident;
    let mut field_conversions = quote! {};
    for field in &data.fields {
        let Some(name) = &field.ident else {
            return Err(syn::Error::new(field.span(), "Pls no tuple struct"));
        };

        field_conversions.append_all(quote! {
            #name: self.#name.intersection(other.#name)?,
        })
    }

    Ok(quote! {
        // TODO: use absolute path
        impl Intersection for #my_type {
            fn intersection(self, other: Self) -> ::core::option::Option<Self> {
                Some(Self {
                    #field_conversions
                })
            }
        }
    })
}
