//! This `proc-macro` crate exports the [`Intersection`] derive macro, see the trait documentation
//! for more information.
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for the [`Intersection`] trait on structs.
#[proc_macro_derive(Intersection)]
pub fn intersection_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    inner::derive(input).into()
}

mod inner {
    use proc_macro2::TokenStream;
    use quote::{quote, TokenStreamExt};
    use syn::{spanned::Spanned, DeriveInput, Error};

    pub(crate) fn derive(input: DeriveInput) -> TokenStream {
        if let syn::Data::Struct(data) = &input.data {
            derive_for_struct(&input, data).unwrap_or_else(Error::into_compile_error)
        } else {
            syn::Error::new(
                input.span(),
                "Deriving `Intersection` is only supported for structs",
            )
            .into_compile_error()
        }
    }

    pub(crate) fn derive_for_struct(
        input: &DeriveInput,
        data: &syn::DataStruct,
    ) -> syn::Result<TokenStream> {
        let my_type = &input.ident;
        let mut field_conversions = quote! {};
        for field in &data.fields {
            let Some(name) = &field.ident else {
                return Err(syn::Error::new(
                    field.span(),
                    "Tuple structs are not currently supported",
                ));
            };

            // TODO(Sebastian): Here, and in the `quote` below, we are referring to `Intersection`
            // with its relative name, which will fail if the user renames the trait
            // when importing, e.g. `use mullvad_types::Intersection as SomethingElse`.
            // This is a know limitation of procural macros (declarative macros can use the `$crate`
            // syntax). If the issue arises then it can be solve using the
            // <https://crates.io/crates/proc-macro-crate> crate. Add it if necessary.
            field_conversions.append_all(quote! {
                #name: Intersection::intersection(self.#name, other.#name)?,
            })
        }

        Ok(quote! {
            impl Intersection for #my_type {
                fn intersection(self, other: Self) -> ::core::option::Option<Self> {
                    ::core::option::Option::Some(Self {
                        #field_conversions
                    })
                }
            }
        })
    }
}
