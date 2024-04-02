//! This `proc-macro` crate exports the [`Intersection`] derive macro, see it's documentation for
//! explanations of how it works.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error};

/// Derive macro for the [`Intersection`] trait on structs.
///
/// The macro applies the intersection on each struct field separately, and returns the resulting
/// type or `None` if any of the intersections failed.
///
/// Derive intersection on [`RelayQuery`]
/// ```rust, ignore
/// #[derive(Intersection)]
/// struct RelayQuery {
///     pub location: Constraint<LocationConstraint>,
///     pub providers: Constraint<Providers>,
///     pub ownership: Constraint<Ownership>,
///     pub tunnel_protocol: Constraint<TunnelType>,
///     pub wireguard_constraints: WireguardRelayQuery,
///     pub openvpn_constraints: OpenVpnRelayQuery,
/// }
/// ```
///
/// produces an implementation like this:
///
/// ```rust, ignore
/// impl Intersection for RelayQuery {
///     fn intersection(self, other: Self) -> Option<Self>
///     where
///         Self: PartialEq,
///         Self: Sized,
///     {
///         Some(RelayQuery {
///             location: self.location.intersection(other.location)?,
///             providers: self.providers.intersection(other.providers)?,
///             ownership: self.ownership.intersection(other.ownership)?,
///             tunnel_protocol: self.tunnel_protocol.intersection(other.tunnel_protocol)?,
///             wireguard_constraints: self
///                 .wireguard_constraints
///                 .intersection(other.wireguard_constraints)?,
///             openvpn_constraints: self
///                 .openvpn_constraints
///                 .intersection(other.openvpn_constraints)?,
///         })
///     }
/// }
/// ```
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
            #name: Intersection::intersection(self.#name, other.#name)?,
        })
    }

    Ok(quote! {
        // TODO: use absolute path
        impl Intersection for #my_type {
            fn intersection(self, other: Self) -> ::core::option::Option<Self> {
                ::core::option::Option::Some(Self {
                    #field_conversions
                })
            }
        }
    })
}
