use app_dirs;
use chrono::{DateTime, Local};
use error_chain::ChainedError;
use futures::Future;

use mullvad_rpc::{HttpHandle, RelayListProxy};
use mullvad_types::location::Location;
use mullvad_types::relay_constraints::{Constraint, LocationConstraint, Match, OpenVpnConstraints,
                                       RelayConstraints, TunnelConstraints};
use mullvad_types::relay_list::{Relay, RelayList, RelayTunnels};

use serde_json;

use talpid_types::net::{TransportProtocol, TunnelEndpoint, TunnelEndpointData};

use std::fs::File;
use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rand::{self, Rng, ThreadRng};
use rand::distributions::{IndependentSample, Range};
use tokio_timer::{TimeoutError, Timer};


error_chain! {
    errors {
        RelayCacheError { description("Error with relay cache on disk") }
        DownloadError { description("Error when trying to download the list of relays") }
        TimeoutError { description("Timed out when trying to download the list of relays") }
        NoRelay { description("No relays matching current constraints") }
        SerializationError { description("Error in serialization of relaylist") }
    }
}

impl<F> From<TimeoutError<F>> for Error {
    fn from(_: TimeoutError<F>) -> Error {
        Error::from_kind(ErrorKind::TimeoutError)
    }
}

pub struct RelaySelector {
    locations: RelayList,
    relays: Vec<Relay>,
    last_updated: SystemTime,
    rng: ThreadRng,
    rpc_client: RelayListProxy<HttpHandle>,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk. Use the `update` method
    /// to refresh the relay list from the internet.
    pub fn new(rpc_handle: HttpHandle, resource_dir: &Path) -> Result<Self> {
        let (last_updated, relay_list) = Self::read_cached_relays(resource_dir)?;
        let (locations, relays) = Self::process_relay_list(relay_list);
        info!(
            "Initialized with {} cached relays from {}",
            relays.len(),
            DateTime::<Local>::from(last_updated).format(::logging::DATE_TIME_FORMAT_STR)
        );
        Ok(RelaySelector {
            locations,
            relays,
            last_updated,
            rng: rand::thread_rng(),
            rpc_client: RelayListProxy::new(rpc_handle),
        })
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_locations(&mut self) -> &RelayList {
        &self.locations
    }

    /// Returns the time when the relay list backing this selector was last fetched from the
    /// internet.
    pub fn get_last_updated(&self) -> SystemTime {
        self.last_updated
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    pub fn get_tunnel_endpoint(
        &mut self,
        constraints: &RelayConstraints,
    ) -> Result<(Relay, TunnelEndpoint)> {
        // Highest priority preference. Where we prefer OpenVPN using UDP. But without changing
        // any constraints that are explicitly specified.
        let tunnel_constraints1 = match constraints.tunnel {
            Constraint::Any => TunnelConstraints::OpenVpn(OpenVpnConstraints {
                port: Constraint::Any,
                protocol: Constraint::Only(TransportProtocol::Udp),
            }),
            Constraint::Only(TunnelConstraints::OpenVpn(ref openvpn_constraints)) => {
                TunnelConstraints::OpenVpn(OpenVpnConstraints {
                    port: openvpn_constraints.port.clone(),
                    protocol: Constraint::Only(
                        openvpn_constraints
                            .protocol
                            .clone()
                            .unwrap_or(TransportProtocol::Udp),
                    ),
                })
            }
            Constraint::Only(ref tunnel_constraints) => tunnel_constraints.clone(),
        };
        let relay_constraints1 = RelayConstraints {
            location: constraints.location.clone(),
            tunnel: Constraint::Only(tunnel_constraints1),
        };

        if let Some((relay, endpoint)) = self.get_tunnel_endpoint_internal(&relay_constraints1) {
            debug!("Relay matched on highest preference");
            Ok((relay, endpoint))
        } else if let Some((relay, endpoint)) = self.get_tunnel_endpoint_internal(constraints) {
            debug!("Relay matched on second preference");
            Ok((relay, endpoint))
        } else {
            bail!(ErrorKind::NoRelay);
        }
    }

    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal(
        &mut self,
        constraints: &RelayConstraints,
    ) -> Option<(Relay, TunnelEndpoint)> {
        let matching_relays: Vec<Relay> = self.relays
            .iter()
            .filter_map(|relay| Self::matching_relay(relay, constraints))
            .collect();

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                info!(
                    "Selected relay {} at {}",
                    selected_relay.hostname, selected_relay.ipv4_addr_in
                );
                self.get_random_tunnel(&selected_relay.tunnels)
                    .map(|tunnel_parameters| {
                        let endpoint = TunnelEndpoint {
                            address: IpAddr::V4(selected_relay.ipv4_addr_in),
                            tunnel: tunnel_parameters,
                        };
                        (selected_relay.clone(), endpoint)
                    })
            })
    }

    /// Takes a `Relay` and a corresponding `RelayConstraints` and returns a new `Relay` if the
    /// given relay matches the constraints.
    fn matching_relay(relay: &Relay, constraints: &RelayConstraints) -> Option<Relay> {
        let matches_location = match constraints.location {
            Constraint::Any => true,
            Constraint::Only(LocationConstraint::Country(ref country)) => {
                relay
                    .location
                    .as_ref()
                    .map_or(false, |loc| loc.country_code == *country)
                    && relay.include_in_country
            }
            Constraint::Only(LocationConstraint::City(ref country, ref city)) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country && loc.city_code == *city
                })
            }
        };
        if !matches_location {
            return None;
        }
        let relay = match constraints.tunnel {
            Constraint::Any => relay.clone(),
            Constraint::Only(ref tunnel_constraints) => {
                let mut relay = relay.clone();
                relay.tunnels = Self::matching_tunnels(&relay.tunnels, tunnel_constraints);
                relay
            }
        };
        if relay.tunnels.openvpn.is_empty() {
            None
        } else {
            Some(relay)
        }
    }

    /// Takes a `RelayTunnels` object which in turn is a collection of tunnel configurations for
    /// a given relay. Then returns a new `RelayTunnels` instance with only the entries that
    /// matches the given `TunnelConstraints`.
    fn matching_tunnels(
        tunnels: &RelayTunnels,
        tunnel_constraints: &TunnelConstraints,
    ) -> RelayTunnels {
        RelayTunnels {
            openvpn: tunnels
                .openvpn
                .iter()
                .filter(|endpoint| tunnel_constraints.matches(*endpoint))
                .cloned()
                .collect(),
            wireguard: tunnels
                .wireguard
                .iter()
                .filter(|endpoint| tunnel_constraints.matches(*endpoint))
                .cloned()
                .collect(),
        }
    }

    /// Pick a random relay from the given slice. Will return `None` if the given slice is empty
    /// or all relays in it has zero weight.
    fn pick_random_relay<'a>(&mut self, relays: &'a [Relay]) -> Option<&'a Relay> {
        let total_weight: u64 = relays.iter().map(|relay| relay.weight).sum();
        debug!(
            "Selecting among {} relays with combined weight {}",
            relays.len(),
            total_weight
        );
        if total_weight == 0 {
            None
        } else {
            // Pick a random number in the range 0 - total_weight. This choses the relay.
            let mut i: u64 = Range::new(0, total_weight + 1).ind_sample(&mut self.rng);
            Some(
                relays
                    .iter()
                    .find(|relay| {
                        i = i.saturating_sub(relay.weight);
                        i == 0
                    })
                    .unwrap(),
            )
        }
    }

    fn get_random_tunnel(&mut self, tunnels: &RelayTunnels) -> Option<TunnelEndpointData> {
        self.rng
            .choose(&tunnels.openvpn)
            .cloned()
            .map(|openvpn_endpoint| TunnelEndpointData::OpenVpn(openvpn_endpoint))
    }

    /// Downloads the latest relay list and caches it. This operation is blocking.
    pub fn update(&mut self, timeout: Duration) -> Result<()> {
        info!("Downloading list of relays...");
        let download_future = self.rpc_client
            .relay_list()
            .map_err(|e| Error::with_chain(e, ErrorKind::DownloadError));
        let relay_list = Timer::default().timeout(download_future, timeout).wait()?;
        if let Err(e) = Self::cache_relays(&relay_list) {
            error!("Unable to save relays to cache: {}", e.display_chain());
        }
        let (locations, relays) = Self::process_relay_list(relay_list);
        info!("Downloaded relay inventory has {} relays", relays.len());
        self.locations = locations;
        self.relays = relays;
        self.last_updated = SystemTime::now();
        Ok(())
    }

    // Extracts all relays from their corresponding cities and return them as a separate vector.
    fn process_relay_list(mut relay_list: RelayList) -> (RelayList, Vec<Relay>) {
        let mut relays = Vec::new();
        for country in &mut relay_list.countries {
            let country_name = country.name.clone();
            let country_code = country.code.clone();
            for city in &mut country.cities {
                city.has_active_relays = !city.relays.is_empty();
                let city_name = city.name.clone();
                let city_code = city.code.clone();
                let latitude = city.latitude;
                let longitude = city.longitude;
                relays.extend(city.relays.drain(..).map(|mut relay| {
                    relay.location = Some(Location {
                        country: country_name.clone(),
                        country_code: country_code.clone(),
                        city: city_name.clone(),
                        city_code: city_code.clone(),
                        latitude,
                        longitude,
                    });
                    relay
                }));
            }
        }
        (relay_list, relays)
    }

    /// Write a `RelayList` to the cache file.
    fn cache_relays(relays: &RelayList) -> Result<()> {
        let file = File::create(Self::get_cache_path()?).chain_err(|| ErrorKind::RelayCacheError)?;
        serde_json::to_writer_pretty(file, relays).chain_err(|| ErrorKind::SerializationError)
    }

    /// Try to read the relays, first from cache and if that fails from the `resource_dir`.
    fn read_cached_relays(resource_dir: &Path) -> Result<(SystemTime, RelayList)> {
        match Self::get_cache_path().and_then(|path| Self::read_relays(&path)) {
            Ok(value) => Ok(value),
            Err(read_cache_error) => match Self::read_relays(&resource_dir.join("relays.json")) {
                Ok(value) => Ok(value),
                Err(read_resource_error) => Err(read_cache_error.chain_err(|| read_resource_error)),
            },
        }
    }

    /// Read and deserialize a `RelayList` from a given path.
    /// Returns the file modification time and the relays.
    fn read_relays(path: &Path) -> Result<(SystemTime, RelayList)> {
        debug!(
            "Trying to read relays cache from {}",
            path.to_string_lossy()
        );
        let (last_modified, file) = Self::read_file(path).chain_err(|| ErrorKind::RelayCacheError)?;
        let relay_list = serde_json::from_reader(file).chain_err(|| ErrorKind::SerializationError)?;
        Ok((last_modified, relay_list))
    }

    fn read_file(path: &Path) -> io::Result<(SystemTime, File)> {
        let file = File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    fn get_cache_path() -> Result<PathBuf> {
        let dir = app_dirs::app_root(app_dirs::AppDataType::UserCache, &::APP_INFO)
            .chain_err(|| ErrorKind::RelayCacheError)?;
        Ok(dir.join("relays.json"))
    }
}
