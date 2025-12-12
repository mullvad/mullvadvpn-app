//! A module dedicated to retrieving the relay list from the Mullvad API.

use crate::rest;

use hyper::{StatusCode, body::Incoming, header};
use mullvad_types::{
    location,
    relay_list::{self, BridgeList, RelayListCountry},
};
use serde::{Deserialize, Serialize};
use talpid_types::net::wireguard;
use vec1::Vec1;

use std::{
    collections::{BTreeMap, HashSet},
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    time::Duration,
};

/// Fetches relay list from <https://api.mullvad.net/app/v1/relays>
#[derive(Clone)]
pub struct RelayListProxy {
    handle: rest::MullvadRestHandle,
}

const RELAY_LIST_TIMEOUT: Duration = Duration::from_secs(15);

impl RelayListProxy {
    /// Construct a new relay list rest client
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    /// Fetch the relay list
    pub fn relay_list(
        &self,
        prev_etag: Option<ETag>,
    ) -> impl Future<Output = Result<Option<CachedRelayList>, rest::Error>> {
        let request = self.relay_list_response(prev_etag.clone());

        async move {
            let response = request.await?;

            match prev_etag {
                Some(_) if response.status() == StatusCode::NOT_MODIFIED => Ok(None),
                _ => {
                    // If the API returns a response, it should contain an ETag.
                    let etag =
                        Self::extract_etag(&response).ok_or(rest::Error::InvalidHeaderError)?;

                    let relay_list: ServerRelayList = response.deserialize().await?;

                    Ok(Some(relay_list.cache(etag)))
                }
            }
        }
    }

    pub fn relay_list_response(
        &self,
        prev_etag: Option<ETag>,
    ) -> impl Future<Output = Result<rest::Response<Incoming>, rest::Error>> {
        let service = self.handle.service.clone();
        let request = self.handle.factory.get("app/v1/relays");

        async move {
            let mut request = request?
                .timeout(RELAY_LIST_TIMEOUT)
                .expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);

            if let Some(ref prev_tag) = prev_etag {
                request = request.header(header::IF_NONE_MATCH, &prev_tag.0)?;
            }

            service.request(request).await
        }
    }

    pub fn extract_etag(response: &rest::Response<Incoming>) -> Option<ETag> {
        response
            .headers()
            .get(header::ETAG)
            .and_then(|s| s.to_str().ok())
            .map(|s| ETag(s.to_owned()))
    }
}

/// Relay list as served by the API.
///
/// This stuct should conform to the API response 1-1.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerRelayList {
    locations: BTreeMap<String, Location>,
    wireguard: Wireguard,
    bridge: Bridges,
}

/// Relay list as served by the API, paired with the corresponding [`ETag`] from the response header.
#[derive(Debug, Deserialize, Serialize)]
pub struct CachedRelayList {
    #[serde(flatten)]
    relay_list: ServerRelayList,
    etag: Option<ETag>,
}

/// An (ETag header)[https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/ETag] returned by the relay list API.
/// The etag is used to version the API response, and is used to check if the response has changed since the last request.
/// This can potentially save some bandwidth, especially important for the server side.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ETag(pub String);

impl ServerRelayList {
    /// Associate this relay list with a specific [ETag].
    fn cache(self, etag: ETag) -> CachedRelayList {
        CachedRelayList {
            relay_list: self,
            etag: Some(etag),
        }
    }

    // Convert a relay list response to internal mullvad types.
    //
    // - `self`: on-disk / network representation
    pub fn into_internal_repr(self) -> (relay_list::RelayList, relay_list::BridgeList) {
        let Self {
            locations,
            wireguard,
            bridge,
        } = self;

        let countries = {
            let mut countries = BTreeMap::new();
            for (code, location) in locations.into_iter() {
                match split_location_code(&code) {
                    Some((country_code, city_code)) => {
                        let country_code = country_code.to_lowercase();
                        let city_code = city_code.to_lowercase();
                        let country = countries
                            .entry(country_code.clone())
                            .or_insert_with(|| location_to_country(&location, country_code));
                        country.cities.push(location_to_city(&location, city_code));
                    }
                    None => {
                        log::error!("Bad location code:{}", code);
                        continue;
                    }
                }
            }
            countries
        };

        // Note: Wireguard::extract_relays needs to be called before Bridges::extract_relays because <TODO>
        let wireguard_endpointdata = wireguard.endpoint_data();
        let countries = wireguard.extract_relays(countries);
        let bridge_list = bridge.extract_relays(&countries);
        let relay_list = relay_list::RelayList {
            wireguard: wireguard_endpointdata,
            countries: countries.into_values().collect(),
        };

        (relay_list, bridge_list)
    }
}

impl CachedRelayList {
    /// Read the [`ETag`] of the cached relay list.
    pub const fn etag(&self) -> Option<&ETag> {
        self.etag.as_ref()
    }

    /// See [`ServerRelayList::into_internal_repr`].
    pub fn into_internal_repr(self) -> (relay_list::RelayList, BridgeList) {
        self.relay_list.into_internal_repr()
    }
}

/// Splits a location code into a country code and a city code. The input is expected to be in a
/// format like `se-mma`, with `se` being the country code, `mma` being the city code.
fn split_location_code(location: &str) -> Option<(&str, &str)> {
    let mut parts = location.split('-');
    let country = parts.next()?;
    let city = parts.next()?;

    Some((country, city))
}

fn location_to_country(location: &Location, code: String) -> relay_list::RelayListCountry {
    relay_list::RelayListCountry {
        cities: vec![],
        name: location.country.clone(),
        code,
    }
}

fn location_to_city(location: &Location, code: String) -> relay_list::RelayListCity {
    relay_list::RelayListCity {
        name: location.city.clone(),
        code,
        latitude: location.latitude,
        longitude: location.longitude,
        relays: vec![],
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Location {
    city: String,
    country: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Relay {
    hostname: String,
    active: bool,
    owned: bool,
    location: String,
    provider: String,
    ipv4_addr_in: Ipv4Addr,
    ipv6_addr_in: Option<Ipv6Addr>,
    weight: u64,
    include_in_country: bool,
}

impl Relay {
    fn into_bridge_mullvad_relay(self, location: location::Location) -> relay_list::Bridge {
        let Self {
            hostname,
            active,
            ipv4_addr_in,
            ipv6_addr_in,
            weight,
            // Since bridges are not a user-facing concept (and is only used for API traffic),
            // we dont really care about their location, who provides them etc. We treat the transport
            // of API traffic as insecure anyway.
            include_in_country: _,
            owned: _,
            location: _,
            provider: _,
        } = self;

        relay_list::Bridge(relay_list::Relay {
            hostname,
            ipv4_addr_in,
            ipv6_addr_in,
            active,
            weight,
            location,
        })
    }

    fn convert_to_lowercase(&mut self) {
        self.hostname = self.hostname.to_lowercase();
        self.location = self.location.to_lowercase();
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Wireguard {
    port_ranges: Vec<(u16, u16)>,
    ipv4_gateway: Ipv4Addr,
    ipv6_gateway: Ipv6Addr,
    /// Shadowsocks port ranges available on all WireGuard relays
    #[serde(default)]
    shadowsocks_port_ranges: Vec<(u16, u16)>,
    relays: Vec<WireGuardRelay>,
}

impl From<&Wireguard> for relay_list::EndpointData {
    fn from(wg: &Wireguard) -> Self {
        Self {
            port_ranges: inclusive_range_from_pair_set(wg.port_ranges.clone()).collect(),
            ipv4_gateway: wg.ipv4_gateway,
            ipv6_gateway: wg.ipv6_gateway,
            shadowsocks_port_ranges: inclusive_range_from_pair_set(
                wg.shadowsocks_port_ranges.clone(),
            )
            .collect(),
            udp2tcp_ports: vec![],
        }
    }
}

fn inclusive_range_from_pair_set<T>(
    set: impl IntoIterator<Item = (T, T)>,
) -> impl Iterator<Item = RangeInclusive<T>> {
    set.into_iter().map(inclusive_range_from_pair)
}

fn inclusive_range_from_pair<T>(pair: (T, T)) -> RangeInclusive<T> {
    RangeInclusive::new(pair.0, pair.1)
}

impl Wireguard {
    /// Consumes `self` and return all its relays to as a map from X to [`RelayListCountry`]
    fn extract_relays(
        self,
        mut countries: BTreeMap<String, RelayListCountry>,
    ) -> BTreeMap<String, RelayListCountry> {
        let relays = self.relays;

        for mut wireguard_relay in relays {
            wireguard_relay.relay.convert_to_lowercase();
            if let Some((country_code, city_code)) =
                split_location_code(&wireguard_relay.relay.location)
                && let Some(country) = countries.get_mut(country_code)
                && let Some(city) = country
                    .cities
                    .iter_mut()
                    .find(|city| city.code == city_code)
            {
                let location = location::Location {
                    country: country.name.clone(),
                    country_code: country.code.clone(),
                    city: city.name.clone(),
                    city_code: city.code.clone(),
                    latitude: city.latitude,
                    longitude: city.longitude,
                };

                let relay = wireguard_relay.into_mullvad_relay(location);
                city.relays.push(relay);
            };
        }

        countries
    }

    fn endpoint_data(&self) -> relay_list::EndpointData {
        relay_list::EndpointData::from(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct WireGuardRelay {
    #[serde(flatten)]
    relay: Relay,
    public_key: wireguard::PublicKey,
    #[serde(default)]
    daita: bool,
    #[serde(default)]
    shadowsocks_extra_addr_in: Vec<IpAddr>,
    #[serde(default)]
    features: Features,
}

impl WireGuardRelay {
    fn into_mullvad_relay(self, location: location::Location) -> relay_list::WireguardRelay {
        // Sanity check that new 'features' key is in sync with the old, superceded keys.
        // TODO: Remove `self.daita` (and this check 👇) when `features` key has been completely
        // rolled out to production.
        if self.features.daita.is_some() {
            debug_assert!(self.daita)
        }

        let relay = relay_list::Relay {
            hostname: self.relay.hostname,
            ipv4_addr_in: self.relay.ipv4_addr_in,
            ipv6_addr_in: self.relay.ipv6_addr_in,
            active: self.relay.active,
            weight: self.relay.weight,
            location,
        };
        let endpoint_data = relay_list::WireguardRelayEndpointData {
            public_key: self.public_key,
            // FIXME: This hack is forward-compatible with 'features' being rolled out.
            //        Should unwrap to 'false' once 'daita' field is removed.
            daita: self.features.daita.map(|_| true).unwrap_or(self.daita),
            shadowsocks_extra_addr_in: HashSet::from_iter(self.shadowsocks_extra_addr_in),
            quic: self.features.quic.map(relay_list::Quic::from),
            lwo: self.features.lwo.is_some(),
        };

        relay_list::WireguardRelay::new(
            false,
            false,
            self.relay.include_in_country,
            self.relay.owned,
            self.relay.provider,
            endpoint_data,
            relay,
        )
    }
}

/// Extra features enabled on some (Wireguard) relay, such as obfuscation daemons or Daita.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
struct Features {
    daita: Option<Daita>,
    quic: Option<Quic>,
    lwo: Option<Lwo>,
}

/// DAITA doesn't have any configuration options (exposed by the API).
///
/// Note, an empty struct is not the same as an empty tuple struct according to serde_json!
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Daita {}

/// Parameters for setting up a QUIC obfuscator (connecting to a masque-proxy running on a relay).
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Quic {
    /// In-addresses for the QUIC obfuscator.
    ///
    /// # Note
    ///
    /// This set must be non-empty.
    ///
    /// The primary IPs of the relay will be included if and only if they are listed here.
    addr_in: Vec1<IpAddr>,
    /// Authorization token
    token: String,
    /// Hostname where masque proxy is hosted
    domain: String,
}

impl From<Quic> for relay_list::Quic {
    fn from(value: Quic) -> Self {
        Self::new(value.addr_in, value.token, value.domain)
    }
}

/// LWO doesn't have any configuration options (exposed by the API).
///
/// Note, an empty struct is not the same as an empty tuple struct according to serde_json!
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Lwo {}

/// Mullvad Bridge servers are used for the Bridge API access method.
///
/// The were previously also used for proxying to traffic OpenVPN servers.
#[derive(Debug, Deserialize, Serialize)]
struct Bridges {
    shadowsocks: Vec<relay_list::ShadowsocksEndpointData>,
    /// The physical bridge servers and generic connnection details.
    relays: Vec<Relay>,
}

impl Bridges {
    /// TODO
    fn extract_relays(
        self,
        countries: &BTreeMap<String, relay_list::RelayListCountry>,
    ) -> BridgeList {
        let relays = self
            .relays
            .into_iter()
            .filter_map(|mut bridge_relay| {
                bridge_relay.convert_to_lowercase();
                if let Some((country_code, city_code)) = split_location_code(&bridge_relay.location)
                    && let Some(country) = countries.get(country_code)
                    && let Some(city) = country.cities.iter().find(|city| city.code == city_code)
                {
                    let location = location::Location {
                        country: country.name.clone(),
                        country_code: country.code.clone(),
                        city: city.name.clone(),
                        city_code: city.code.clone(),
                        latitude: city.latitude,
                        longitude: city.longitude,
                    };

                    let relay = bridge_relay.into_bridge_mullvad_relay(location);
                    Some(relay)
                } else {
                    None
                }
            })
            .collect();

        BridgeList {
            bridges: relays,
            bridge_endpoint: relay_list::BridgeEndpointData {
                shadowsocks: self.shadowsocks,
            },
        }
    }
}
