pub mod access_method;
pub mod account;
pub mod auth_failed;
pub mod constraints;
pub mod custom_list;
pub mod device;
pub mod endpoint;
pub mod location;
pub mod relay_constraints;
pub mod relay_list;
pub mod settings;
pub mod states;
pub mod version;
pub mod wireguard;

mod custom_tunnel;
pub use crate::custom_tunnel::*;

// b"mole" is [ 0x6d, 0x6f 0x6c, 0x65 ]
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x6d6f6c65;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;

/// The intersection of two sets of criteria on [`Relay`](crate::relay_list::Relay)s is another
/// criteria which matches the given relay iff both of the original criteria matched. It is
/// primarily used by the relay selector to check whether a given connection method is compatible
/// with the users settings.
///
/// # Examples
///
/// The [`Intersection`] implementation of [`RelayQuery`] upholds the following properties:
///
/// * If two [`RelayQuery`]s differ such that no relay matches both, [`Option::None`] is returned:
/// ```rust
/// # use mullvad_relay_selector::query::builder::RelayQueryBuilder;
/// # use mullvad_types::Intersection;
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
/// # use mullvad_types::Intersection;
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
///
/// The macro recursively applies the intersection on each field of the struct and returns the
/// resulting type or `None` if any of the intersections failed to overlap.
///
/// The macro requires the types of each field to also implement [`Intersection`], which may be done
/// using this derive macro, the
///
/// # Implementing [`Intersection`]
///
/// For structs where each field already implements `Intersection`, the easiest way to implement the
/// trait is using the derive macro. Using the derive macro on [`RelayQuery`]
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
///
/// For types that cannot "overlap", e.g. they only intersect if they are equal, the declarative
/// macro [`impl_intersection_partialeq`] can be used.
///
/// For less trivial cases, the trait needs to be implemented manually. When doing so, make sure
/// that the following properties are upheld:
///
/// - idempotency (if there is an identity element)
/// - commutativity
/// - associativity
pub trait Intersection: Sized {
    fn intersection(self, other: Self) -> Option<Self>;
}

pub use intersection_derive::Intersection;

#[macro_export]
macro_rules! impl_intersection_partialeq {
    ($ty:ty) => {
        impl $crate::Intersection for $ty {
            fn intersection(self, other: Self) -> Option<Self> {
                if self == other {
                    Some(self)
                } else {
                    None
                }
            }
        }
    };
}

impl_intersection_partialeq!(u16);
impl_intersection_partialeq!(bool);

// NOTE: this implementation does not do what you may expect of an intersection
impl_intersection_partialeq!(relay_constraints::Providers);
// NOTE: should take actual intersection
impl_intersection_partialeq!(relay_constraints::LocationConstraint);
impl_intersection_partialeq!(relay_constraints::Ownership);
// NOTE: it contains an inner constraint
impl_intersection_partialeq!(relay_constraints::TransportPort);
impl_intersection_partialeq!(talpid_types::net::TransportProtocol);
impl_intersection_partialeq!(talpid_types::net::TunnelType);
impl_intersection_partialeq!(talpid_types::net::IpVersion);
