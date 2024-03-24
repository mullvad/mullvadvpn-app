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
/// ```rust
/// #[derive(Intersection)]
/// struct RelayQuery {
///     ...
/// }
/// ```
///
/// produces an implementation like this:
///
/// ```rust
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
///
/// * If two [`RelayQuery`]s differ such that no relay matches both, [`Option::None`] is returned:
/// ```rust
/// # use mullvad_relay_selector::query::builder::RelayQueryBuilder;
/// # use crate::mullvad_relay_selector::query::Intersection;
/// let query_a = RelayQueryBuilder::new().wireguard().build();
/// let query_b = RelayQueryBuilder::new().openvpn().build();
/// assert_eq!(query_a.intersection(query_b), None);
/// ```
///
/// * Otherwise, a new [`RelayQuery`] is returned where each constraint is
/// as specific as possible. See [`Constraint`] for further details.
/// ```rust
/// # use crate::mullvad_relay_selector::*;
/// # use crate::mullvad_relay_selector::query::*;
/// # use crate::mullvad_relay_selector::query::builder::*;
/// # use mullvad_types::relay_list::*;
/// # use talpid_types::net::wireguard::PublicKey;
///
/// // The relay list used by `relay_selector` in this example
/// let relay_list = RelayList {
/// #   etag: None,
/// #   openvpn: OpenVpnEndpointData { ports: vec![] },
/// #   bridge: BridgeEndpointData {
/// #       shadowsocks: vec![],
/// #   },
/// #   wireguard: WireguardEndpointData {
/// #       port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
/// #       ipv4_gateway: "10.64.0.1".parse().unwrap(),
/// #       ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
/// #       udp2tcp_ports: vec![],
/// #   },
///     countries: vec![RelayListCountry {
///         name: "Sweden".to_string(),
/// #       code: "Sweden".to_string(),
///         cities: vec![RelayListCity {
///             name: "Gothenburg".to_string(),
/// #           code: "Gothenburg".to_string(),
/// #           latitude: 57.70887,
/// #           longitude: 11.97456,
///             relays: vec![Relay {
///                 hostname: "se9-wireguard".to_string(),
///                 ipv4_addr_in: "185.213.154.68".parse().unwrap(),
/// #               ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
/// #               include_in_country: false,
/// #               active: true,
/// #               owned: true,
/// #               provider: "31173".to_string(),
/// #               weight: 1,
/// #               endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
/// #                   public_key: PublicKey::from_base64(
/// #                       "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
/// #                   )
/// #                   .unwrap(),
/// #               }),
/// #               location: None,
///             }],
///         }],
///     }],
/// };
///
/// # let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list.clone());
/// # let city = |country, city| GeographicLocationConstraint::city(country, city);
///
/// let query_a = RelayQueryBuilder::new().wireguard().build();
/// let query_b = RelayQueryBuilder::new().location(city("Sweden", "Gothenburg")).build();
///
/// let result = relay_selector.get_relay_by_query(query_a.intersection(query_b).unwrap());
/// assert!(result.is_ok());
/// ```
///
/// This way, if the mullvad app wants to check if the user's relay settings
/// are compatible with any other [`RelayQuery`], for examples those defined by
/// [`RETRY_ORDER`] , taking the intersection between them will never result in
/// a situation where the app can override the user's preferences.
///
/// [`RETRY_ORDER`]: crate::RETRY_ORDER
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
