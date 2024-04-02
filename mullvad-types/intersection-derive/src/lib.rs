//! This `proc-macro` crate exports the [`Intersection`] derive macro, see it's documentation for
//! explanations of how it works.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error};

/// Derive macro for the [`Intersection`] trait on structs.
#[proc_macro_derive(Intersection)]
pub fn intersection_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => derive_for_struct(&input, data),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn derive_for_struct(input: &DeriveInput, data: &syn::DataStruct) -> syn::Result<TokenStream2> {
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
