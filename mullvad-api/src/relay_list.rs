//! A module dedicated to retrieving the relay list from the Mullvad API.

use crate::{relay_list_transparency, rest};

use hyper::{StatusCode, body::Incoming};
use mullvad_types::{
    location,
    relay_list::{self, BridgeList, RelayListCountry},
};
use serde::{Deserialize, Serialize};
use talpid_types::net::wireguard;
use vec1::Vec1;

use crate::relay_list_transparency::{RelayListDigest, RelayListSignature, Sha256Bytes};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
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

    /// Fetches and verifies the Sigsum transparency logged relay list.
    /// Currently, verification failures are only logged, and do not cause this function to return
    /// an error. An error will only be returned if the HTTP request failed or relay list is
    /// corrupted in some way.
    pub async fn relay_list(
        &self,
        latest_digest: Option<RelayListDigest>,
        latest_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Option<CachedRelayList>, rest::Error> {
        relay_list_transparency::download_and_verify_relay_list(
            self,
            latest_digest,
            latest_timestamp,
            &self.handle.sigsum_trusted_pubkeys,
        )
        .await?
        .map(|rl| {
            let relay_list: ServerRelayList = serde_json::from_slice(&rl.content)?;
            Ok(relay_list.cache(rl.digest, rl.timestamp))
        })
        .transpose()
    }

    /// Fetch the relay list
    pub(crate) async fn relay_list_content(
        &self,
        digest: &RelayListDigest,
    ) -> Result<RelayListResponse, rest::Error> {
        let response = self.relay_list_content_response(digest).await?;

        response
            .body()
            .await
            .map(|body| {
                let digest: Sha256Bytes = Sha256::digest(&body).into();
                let relay_list_digest = RelayListDigest::new(hex::encode(digest));
                RelayListResponse {
                    content: body,
                    digest: relay_list_digest,
                }
            })
            .inspect_err(|_err| log::error!("Failed to fetch relay list"))
    }

    async fn relay_list_content_response(
        &self,
        digest: &RelayListDigest,
    ) -> Result<rest::Response<Incoming>, rest::Error> {
        let service = self.handle.service.clone();
        let request = self.handle.factory.get(&format!("trl/v0/data/{digest}"));

        let request = request?
            .timeout(RELAY_LIST_TIMEOUT)
            .expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);

        service.request(request).await
    }

    /// Fetch the relay list sigsum timestamp
    pub(crate) async fn relay_list_latest_timestamp(
        &self,
    ) -> Result<RelayListSignature, rest::Error> {
        let response = self.relay_list_timestamp_response().await?;

        let relay_list_sigsum = response
            .body()
            .await
            .and_then(|body| {
                str::from_utf8(&body)
                    .map_err(|_| rest::Error::InvalidUtf8Error)
                    .and_then(RelayListSignature::parse)
            })
            .inspect_err(|_err| {
                log::error!("Failed to deserialize API response of relay list sigsum")
            })?;

        Ok(relay_list_sigsum)
    }

    async fn relay_list_timestamp_response(&self) -> Result<rest::Response<Incoming>, rest::Error> {
        let service = self.handle.service.clone();
        let request = self.handle.factory.get("trl/v0/timestamps/latest");

        let request = request?
            .timeout(RELAY_LIST_TIMEOUT)
            .expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);

        service.request(request).await
    }
}
/// The unparsed relay list bytes together with a digest of the content.
#[derive(Debug)]
pub struct RelayListResponse {
    pub content: Vec<u8>,
    pub digest: RelayListDigest,
}

/// Relay list as served by the API.
///
/// This struct should conform to the API response 1-1.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerRelayList {
    locations: BTreeMap<String, Location>,
    wireguard: Wireguard,
    bridge: Bridges,
}

/// Relay list as served by the API, paired with the corresponding sigsum digest and timestamp.
/// TODO: we need to either use #[serde(default)] for `digest` and `timestamp` or write a
/// migration that changes any previously saved cached relay list. Which is better?
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedRelayList {
    #[serde(flatten)]
    relay_list: ServerRelayList,

    /// The digest (Sha256 hash) of the relay list content. This needs to be cached in order to
    /// determine if a new relay list fetch is needed or not. If the digest that is returned
    /// from the sigsum timestamp matches this digest, there is no new relay list that needs to
    /// be fetched.
    #[serde(default)]
    digest: RelayListDigest,

    /// The timestamp of when the relay list was signed. This is needed to check that the timestamp
    /// we get form the API is not older than this value.
    #[serde(default)]
    timestamp: DateTime<Utc>,
}

impl ServerRelayList {
    /// Associate this relay list with a specific [`RelayListDigest`].
    const fn cache(self, digest: RelayListDigest, timestamp: DateTime<Utc>) -> CachedRelayList {
        CachedRelayList {
            relay_list: self,
            digest,
            timestamp,
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

        let wireguard_endpointdata = {
            const UDP2TCP_PORTS: [u16; 3] = [80, 443, 5001];
            let mut data = wireguard.endpoint_data();
            // Append data for obfuscation protocols ourselves, since the API does not provide it.
            if data.udp2tcp_ports.is_empty() {
                data.udp2tcp_ports.extend(UDP2TCP_PORTS);
            }

            data
        };
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
    /// Read the [`RelayListDigest`] of the cached relay list.
    pub const fn digest(&self) -> &RelayListDigest {
        &self.digest
    }

    pub const fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Location {
    city: String,
    country: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    /// Consumes `self` and group all relays with their geographical location, keyed by country code.
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
/// The were previously also used for proxying traffic to OpenVPN servers.
///
/// See [Relay] for details.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Bridges {
    shadowsocks: Vec<relay_list::ShadowsocksEndpointData>,
    /// The physical bridge servers and generic connnection details.
    relays: Vec<Relay>,
}

impl Bridges {
    /// Pluck out all servers from a relay list.
    ///
    /// See [Bridges] for details.
    fn extract_relays(
        self,
        countries: &BTreeMap<String, relay_list::RelayListCountry>,
    ) -> BridgeList {
        let bridges = self
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

                    Some(bridge_relay.into_bridge_mullvad_relay(location))
                } else {
                    None
                }
            })
            .collect();

        BridgeList {
            bridges,
            bridge_endpoint: relay_list::BridgeEndpointData {
                shadowsocks: self.shadowsocks,
            },
        }
    }
}
