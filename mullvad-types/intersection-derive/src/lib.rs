//! This `proc-macro` crate exports the [`Intersection`] derive macro, see the trait documentation
//! for more information.
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for the [`Intersection`] trait on structs and enums.
///
/// ## Structs
///
/// For structs with named fields, the intersection is computed field-by-field.
/// Every field type must implement `Intersection`. If any field's intersection
/// returns `None`, the whole result is `None`.
///
/// ## Enums
///
/// For enums, the intersection is defined variant-by-variant:
///
/// - **Same variant, same variant:** the inner fields are intersected. For unit
///   variants this is always `Some(Variant)`. For newtype variants `V(T)`,
///   this is `Some(V(a.intersection(b)?))`.
/// - **Different variants:** always `None`.
#[proc_macro_derive(Intersection)]
pub fn intersection_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    inner::derive(input).into()
}

mod inner {
    use proc_macro2::TokenStream;
    use quote::{TokenStreamExt, quote};
    use syn::{DeriveInput, Error, spanned::Spanned};

    pub(crate) fn derive(input: DeriveInput) -> TokenStream {
        match &input.data {
            syn::Data::Struct(data) => {
                derive_for_struct(&input, data).unwrap_or_else(Error::into_compile_error)
            }
            syn::Data::Enum(data) => {
                derive_for_enum(&input, data).unwrap_or_else(Error::into_compile_error)
            }
            syn::Data::Union(_) => syn::Error::new(
                input.span(),
                "Deriving `Intersection` is not supported for unions",
            )
            .into_compile_error(),
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

    pub(crate) fn derive_for_enum(
        input: &DeriveInput,
        data: &syn::DataEnum,
    ) -> syn::Result<TokenStream> {
        let my_type = &input.ident;
        let mut match_arms = quote! {};

        for variant in &data.variants {
            let variant_name = &variant.ident;

            match &variant.fields {
                // Unit variant: `Variant ∩ Variant = Some(Variant)`
                syn::Fields::Unit => {
                    match_arms.append_all(quote! {
                        (Self::#variant_name, Self::#variant_name) => {
                            ::core::option::Option::Some(Self::#variant_name)
                        }
                    });
                }
                // Newtype variant: `V(a) ∩ V(b) = Some(V(a.intersection(b)?))`
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.len() != 1 {
                        return Err(syn::Error::new(
                            variant.span(),
                            "Deriving `Intersection` on enums only supports unit and \
                             single-field tuple variants",
                        ));
                    }
                    match_arms.append_all(quote! {
                        (Self::#variant_name(__a), Self::#variant_name(__b)) => {
                            ::core::option::Option::Some(
                                Self::#variant_name(Intersection::intersection(__a, __b)?)
                            )
                        }
                    });
                }
                syn::Fields::Named(_) => {
                    return Err(syn::Error::new(
                        variant.span(),
                        "Deriving `Intersection` on enums does not support struct variants",
                    ));
                }
            }
        }

        // Wildcard: different variants → None
        match_arms.append_all(quote! {
            _ => ::core::option::Option::None,
        });

        Ok(quote! {
            impl Intersection for #my_type {
                fn intersection(self, other: Self) -> ::core::option::Option<Self> {
                    match (self, other) {
                        #match_arms
                    }
                }
            }
        })
    }
}
