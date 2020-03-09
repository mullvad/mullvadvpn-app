/// A module dedicated to retrieving the relay list from the master API.
use crate::rest;

use hyper::{Method, StatusCode};
use mullvad_types::{location, relay_list};
use talpid_types::net::wireguard;

use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, Ipv6Addr},
};

/// Fetches relay list from https://api.mullvad.net/v1/relays
pub struct RelayListProxy {
    handle: rest::MullvadRestHandle,
}

impl RelayListProxy {
    /// Construct a new relay list rest client
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    /// Fetch the relay list
    pub fn relay_list_v3(
        &self,
    ) -> impl futures01::future::Future<Item = relay_list::RelayList, Error = rest::Error> {
        let service = self.handle.service.clone();
        let request = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/relays",
            Method::GET,
            None,
            StatusCode::OK,
        );

        self.handle.service.compat_spawn(async move {
            let response: ServerRelayList = rest::deserialize_body(request.await?).await?;
            Ok(response.into_relay_list())
        })
    }
}


#[derive(Debug, serde::Deserialize)]
struct ServerRelayList {
    locations: BTreeMap<String, Location>,
    openvpn: OpenVpn,
    wireguard: Wireguard,
    bridge: Bridges,
}

impl ServerRelayList {
    fn into_relay_list(self) -> relay_list::RelayList {
        let mut countries = BTreeMap::new();
        let Self {
            locations,
            openvpn,
            wireguard,
            bridge,
        } = self;

        for (code, location) in locations.into_iter() {
            match split_location_code(&code) {
                Some((country_code, city_code)) => {
                    let country = countries
                        .entry(country_code.to_string())
                        .or_insert_with(|| location_to_country(&location, country_code.to_owned()));
                    country
                        .cities
                        .push(location_to_city(&location, city_code.to_owned()));
                }
                None => {
                    log::error!("Bad location code - {}", code);
                    continue;
                }
            }
        }


        Self::add_openvpn_relays(&mut countries, openvpn);
        Self::add_wireguard_relays(&mut countries, wireguard);
        Self::add_bridge_relays(&mut countries, bridge);


        relay_list::RelayList {
            countries: countries
                .into_iter()
                .map(|(_key, country)| country)
                .collect(),
        }
    }

    fn add_openvpn_relays(
        countries: &mut BTreeMap<String, relay_list::RelayListCountry>,
        openvpn: OpenVpn,
    ) {
        let openvpn_endpoint_data = openvpn.ports;
        for openvpn_relay in openvpn.relays.into_iter() {
            if let Some((country_code, city_code)) = split_location_code(&openvpn_relay.location) {
                if let Some(country) = countries.get_mut(country_code) {
                    if let Some(city) = country
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
                        match city
                            .relays
                            .iter_mut()
                            .find(|r| r.hostname == openvpn_relay.hostname)
                        {
                            Some(relay) => relay.tunnels.openvpn = openvpn_endpoint_data.clone(),
                            None => {
                                let mut relay = relay(openvpn_relay, location);
                                relay.tunnels.openvpn = openvpn_endpoint_data.clone();
                                city.relays.push(relay);
                            }
                        };
                    }
                };
            }
        }
    }

    fn add_wireguard_relays(
        countries: &mut BTreeMap<String, relay_list::RelayListCountry>,
        wireguard: Wireguard,
    ) {
        let Wireguard {
            port_ranges,
            ipv4_gateway,
            ipv6_gateway,
            relays,
        } = wireguard;

        let wireguard_endpoint_data =
            |public_key: wireguard::PublicKey| relay_list::WireguardEndpointData {
                port_ranges: port_ranges.clone(),
                ipv4_gateway,
                ipv6_gateway,
                public_key,
            };

        for wireguard_relay in relays {
            if let Some((country_code, city_code)) =
                split_location_code(&wireguard_relay.relay.location)
            {
                if let Some(country) = countries.get_mut(country_code) {
                    if let Some(city) = country
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
                        match city
                            .relays
                            .iter_mut()
                            .find(|r| r.hostname == wireguard_relay.relay.hostname)
                        {
                            Some(relay) => relay
                                .tunnels
                                .wireguard
                                .push(wireguard_endpoint_data(wireguard_relay.public_key)),
                            None => {
                                let mut relay = relay(wireguard_relay.relay, location);
                                relay.ipv6_addr_in = Some(wireguard_relay.ipv6_addr_in);
                                relay.tunnels.wireguard =
                                    vec![wireguard_endpoint_data(wireguard_relay.public_key)];
                                city.relays.push(relay);
                            }
                        };
                    }
                };
            }
        }
    }

    fn add_bridge_relays(
        countries: &mut BTreeMap<String, relay_list::RelayListCountry>,
        bridges: Bridges,
    ) {
        let Bridges {
            relays,
            shadowsocks,
        } = bridges;

        for bridge_relay in relays {
            if let Some((country_code, city_code)) = split_location_code(&bridge_relay.location) {
                if let Some(country) = countries.get_mut(country_code) {
                    if let Some(city) = country
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

                        match city
                            .relays
                            .iter_mut()
                            .find(|r| r.hostname == bridge_relay.hostname)
                        {
                            Some(relay) => {
                                relay.bridges.shadowsocks = shadowsocks.clone();
                            }
                            None => {
                                let mut relay = relay(bridge_relay, location);
                                relay.bridges.shadowsocks = shadowsocks.clone();
                                city.relays.push(relay);
                            }
                        };
                    }
                };
            }
        }
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
        cities: Vec::with_capacity(20),
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
        relays: Vec::with_capacity(100),
    }
}

fn relay(relay: Relay, location: location::Location) -> relay_list::Relay {
    relay_list::Relay {
        hostname: relay.hostname,
        ipv4_addr_in: relay.ipv4_addr_in,
        ipv6_addr_in: None,
        include_in_country: relay.include_in_country,
        active: relay.active,
        owned: relay.owned,
        provider: relay.provider,
        weight: relay.weight,
        tunnels: Default::default(),
        bridges: Default::default(),
        location: Some(location),
    }
}

#[derive(Debug, serde::Deserialize)]
struct Location {
    city: String,
    country: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, serde::Deserialize)]
struct OpenVpn {
    ports: Vec<relay_list::OpenVpnEndpointData>,
    relays: Vec<Relay>,
}

#[derive(Debug, serde::Deserialize)]
struct Relay {
    hostname: String,
    active: bool,
    owned: bool,
    location: String,
    provider: String,
    ipv4_addr_in: Ipv4Addr,
    weight: u64,
    include_in_country: bool,
}

#[derive(Debug, serde::Deserialize)]
struct Wireguard {
    port_ranges: Vec<(u16, u16)>,
    ipv4_gateway: Ipv4Addr,
    ipv6_gateway: Ipv6Addr,
    relays: Vec<WireGuardRelay>,
}

#[derive(Debug, serde::Deserialize)]
struct WireGuardRelay {
    #[serde(flatten)]
    relay: Relay,
    ipv6_addr_in: Ipv6Addr,
    public_key: wireguard::PublicKey,
}

#[derive(Debug, serde::Deserialize)]
struct Bridges {
    shadowsocks: Vec<relay_list::ShadowsocksEndpointData>,
    relays: Vec<Relay>,
}
