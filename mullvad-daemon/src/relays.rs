use chrono::{DateTime, Local};
use error_chain::ChainedError;
use futures::Future;

use mullvad_rpc::{HttpHandle, RelayListProxy};
use mullvad_types::{
    endpoint::MullvadEndpoint,
    location::Location,
    relay_constraints::{
        Constraint, LocationConstraint, Match, OpenVpnConstraints, RelayConstraints,
        TunnelConstraints, WireguardConstraints,
    },
    relay_list::{Relay, RelayList, RelayTunnels, WireguardEndpointData},
};

use serde_json;

use talpid_types::net::{all_of_the_internet, wireguard, TransportProtocol};

use std::{
    fs::File,
    io,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex, MutexGuard},
    thread,
    time::{self, Duration, SystemTime},
};

use log::{debug, error, info, warn};
use rand::{self, Rng, ThreadRng};
use tokio_timer::{TimeoutError, Timer};

const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";
const RELAYS_FILENAME: &str = "relays.json";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);
/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 2);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_secs(3600);

error_chain! {
    errors {
        RelayCacheError { description("Error with relay cache on disk") }
        DownloadError { description("Error when trying to download the list of relays") }
        DownloadTimeoutError { description("Timed out when trying to download the list of relays") }
        NoRelay { description("No relays matching current constraints") }
        SerializationError { description("Error in serialization of relaylist") }
    }
}

impl<F> From<TimeoutError<F>> for Error {
    fn from(_: TimeoutError<F>) -> Error {
        Error::from_kind(ErrorKind::DownloadTimeoutError)
    }
}

struct ParsedRelays {
    last_updated: SystemTime,
    locations: RelayList,
    relays: Vec<Relay>,
}

impl ParsedRelays {
    pub fn empty() -> Self {
        ParsedRelays {
            last_updated: time::UNIX_EPOCH,
            locations: RelayList::empty(),
            relays: Vec::new(),
        }
    }

    pub fn from_relay_list(mut relay_list: RelayList, last_updated: SystemTime) -> Self {
        let mut relays = Vec::new();
        for country in &mut relay_list.countries {
            let country_name = country.name.clone();
            let country_code = country.code.clone();
            for city in &mut country.cities {
                let city_name = city.name.clone();
                let city_code = city.code.clone();
                let latitude = city.latitude;
                let longitude = city.longitude;
                city.relays = city
                    .relays
                    .iter()
                    .filter(|relay| !relay.tunnels.is_empty())
                    .cloned()
                    .collect();
                for relay in &mut city.relays {
                    let mut relay_with_location = relay.clone();
                    relay_with_location.location = Some(Location {
                        country: country_name.clone(),
                        country_code: country_code.clone(),
                        city: city_name.clone(),
                        city_code: city_code.clone(),
                        latitude,
                        longitude,
                    });
                    relays.push(relay_with_location);
                    relay.tunnels.clear();
                }
            }
        }
        ParsedRelays {
            last_updated,
            locations: relay_list,
            relays,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        debug!("Reading relays from {}", path.as_ref().display());
        let (last_modified, file) =
            Self::open_file(path.as_ref()).chain_err(|| ErrorKind::RelayCacheError)?;
        let relay_list = serde_json::from_reader(io::BufReader::new(file))
            .chain_err(|| ErrorKind::SerializationError)?;

        Ok(Self::from_relay_list(relay_list, last_modified))
    }

    fn open_file(path: &Path) -> io::Result<(SystemTime, File)> {
        let file = File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn locations(&self) -> &RelayList {
        &self.locations
    }

    pub fn relays(&self) -> &Vec<Relay> {
        &self.relays
    }
}

pub struct RelaySelector {
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    rng: ThreadRng,
    updater: RelayListUpdaterHandle,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk. Use the `update` method
    /// to refresh the relay list from the internet.
    pub fn new(rpc_handle: HttpHandle, resource_dir: &Path, cache_dir: &Path) -> Self {
        let cache_path = cache_dir.join(RELAYS_FILENAME);
        let resource_path = resource_dir.join(RELAYS_FILENAME);
        let unsynchronized_parsed_relays = Self::read_cached_relays(&cache_path, &resource_path)
            .unwrap_or_else(|error| {
                let chained_error = error.chain_err(|| "Unable to load cached relays");
                error!("{}", chained_error.display_chain());
                ParsedRelays::empty()
            });
        info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().len(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );
        let parsed_relays = Arc::new(Mutex::new(unsynchronized_parsed_relays));
        let updater = RelayListUpdater::spawn(rpc_handle, cache_path, parsed_relays.clone());
        RelaySelector {
            parsed_relays,
            rng: rand::thread_rng(),
            updater,
        }
    }

    /// Download the newest relay list.
    pub fn update(&self) {
        self.updater
            .send(())
            .expect("Relay list updated thread has stopped unexpectedly");
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_locations(&mut self) -> RelayList {
        self.lock_parsed_relays().locations().clone()
    }

    fn lock_parsed_relays(&self) -> MutexGuard<ParsedRelays> {
        self.parsed_relays
            .lock()
            .expect("Relay updater thread crashed while it held a lock to the list of relays")
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    pub fn get_tunnel_endpoint(
        &mut self,
        constraints: &RelayConstraints,
        retry_attempt: u32,
    ) -> Result<(Relay, MullvadEndpoint)> {
        let preferred_constraints = Self::preferred_constraints(constraints, retry_attempt);
        if let Some((relay, endpoint)) = self.get_tunnel_endpoint_internal(&preferred_constraints) {
            debug!(
                "Relay matched on highest preference for retry attempt {}",
                retry_attempt
            );
            Ok((relay, endpoint))
        } else if let Some((relay, endpoint)) = self.get_tunnel_endpoint_internal(constraints) {
            debug!(
                "Relay matched on second preference for retry attempt {}",
                retry_attempt
            );
            Ok((relay, endpoint))
        } else {
            warn!("No relays matching {}", constraints);
            bail!(ErrorKind::NoRelay);
        }
    }

    fn preferred_constraints(
        original_constraints: &RelayConstraints,
        retry_attempt: u32,
    ) -> RelayConstraints {
        // Prefer UDP by default. But if that has failed a couple of times, then try TCP port 443,
        // which works for many with UDP problems. After that, just alternate between protocols.
        let (preferred_port, preferred_protocol) = match retry_attempt {
            0 | 1 => (Constraint::Any, TransportProtocol::Udp),
            2 | 3 => (Constraint::Only(443), TransportProtocol::Tcp),
            attempt if attempt % 2 == 0 => (Constraint::Any, TransportProtocol::Udp),
            _ => (Constraint::Any, TransportProtocol::Tcp),
        };

        // Highest priority preference. Where we prefer OpenVPN using UDP. But without changing
        // any constraints that are explicitly specified.
        let tunnel_constraints = match original_constraints.tunnel {
            // No constraints, we use our preferred ones.
            Constraint::Any => TunnelConstraints::OpenVpn(OpenVpnConstraints {
                port: preferred_port,
                protocol: Constraint::Only(preferred_protocol),
            }),
            Constraint::Only(TunnelConstraints::OpenVpn(ref openvpn_constraints)) => {
                match openvpn_constraints {
                    // Constrained to OpenVpn, but port/protocol not constrained. Use our preferred.
                    OpenVpnConstraints {
                        port: Constraint::Any,
                        protocol: Constraint::Any,
                    } => TunnelConstraints::OpenVpn(OpenVpnConstraints {
                        port: preferred_port,
                        protocol: Constraint::Only(preferred_protocol),
                    }),
                    // Other constraints, use the original constraints.
                    openvpn_constraints => TunnelConstraints::OpenVpn(openvpn_constraints.clone()),
                }
            }
            // Non-OpenVPN constraints. Respect and keep those constraints.
            Constraint::Only(ref tunnel_constraints) => tunnel_constraints.clone(),
        };
        RelayConstraints {
            location: original_constraints.location.clone(),
            tunnel: Constraint::Only(tunnel_constraints),
        }
    }

    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal(
        &mut self,
        constraints: &RelayConstraints,
    ) -> Option<(Relay, MullvadEndpoint)> {
        let matching_relays: Vec<Relay> = self
            .lock_parsed_relays()
            .relays()
            .iter()
            .filter_map(|relay| Self::matching_relay(relay, constraints))
            .collect();

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                info!(
                    "Selected relay {} at {}",
                    selected_relay.hostname, selected_relay.ipv4_addr_in
                );
                self.get_random_tunnel(&selected_relay, &constraints.tunnel)
                    .map(|endpoint| (selected_relay.clone(), endpoint))
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
            Constraint::Only(LocationConstraint::Hostname(ref country, ref city, ref hostname)) => {
                relay.location.as_ref().map_or(false, |loc| {
                    loc.country_code == *country
                        && loc.city_code == *city
                        && relay.hostname == *hostname
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
                relay.tunnels = Self::matching_tunnels(&mut relay.tunnels, tunnel_constraints);
                relay
            }
        };
        let relay_matches = match constraints.tunnel {
            Constraint::Any => {
                !relay.tunnels.openvpn.is_empty() || !relay.tunnels.wireguard.is_empty()
            }
            Constraint::Only(TunnelConstraints::OpenVpn(_)) => !relay.tunnels.openvpn.is_empty(),
            Constraint::Only(TunnelConstraints::Wireguard(_)) => {
                !relay.tunnels.wireguard.is_empty()
            }
        };

        if relay_matches {
            Some(relay)
        } else {
            None
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
            let mut i: u64 = self.rng.gen_range(0, total_weight + 1);
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

    fn get_random_tunnel(
        &mut self,
        relay: &Relay,
        constraints: &Constraint<TunnelConstraints>,
    ) -> Option<MullvadEndpoint> {
        match constraints {
            // TODO: Handle Constraint::Any case by selecting from both openvpn and wireguard
            // tunnels once wireguard is mature enough
            Constraint::Only(TunnelConstraints::OpenVpn(_)) | Constraint::Any => self
                .rng
                .choose(&relay.tunnels.openvpn)
                .cloned()
                .map(|endpoint| endpoint.into_mullvad_endpoint(relay.ipv4_addr_in.into())),
            Constraint::Only(TunnelConstraints::Wireguard(wg_constraints)) => self
                .rng
                .choose(&relay.tunnels.wireguard)
                .cloned()
                .and_then(|wg_tunnel| {
                    self.wg_data_to_endpoint(relay.ipv4_addr_in.into(), wg_tunnel, wg_constraints)
                }),
        }
    }

    fn wg_data_to_endpoint(
        &mut self,
        host: IpAddr,
        data: WireguardEndpointData,
        constraints: &WireguardConstraints,
    ) -> Option<MullvadEndpoint> {
        let port = self.get_port_for_wireguard_relay(&data, constraints)?;
        let peer_config = wireguard::PeerConfig {
            public_key: data.public_key,
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
        };
        Some(MullvadEndpoint::Wireguard {
            peer: peer_config,
            ipv4_gateway: data.ipv4_gateway,
            ipv6_gateway: data.ipv6_gateway,
        })
    }

    fn get_port_for_wireguard_relay(
        &mut self,
        data: &WireguardEndpointData,
        constraints: &WireguardConstraints,
    ) -> Option<u16> {
        match constraints.port {
            Constraint::Any => {
                let get_port_amount =
                    |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
                let port_amount: u64 = data.port_ranges.iter().map(get_port_amount).sum();

                if port_amount < 1 {
                    return None;
                }

                let mut port_index = self.rng.gen_range(0, port_amount);

                for range in data.port_ranges.iter() {
                    let ports_in_range = get_port_amount(range);
                    if port_index < ports_in_range {
                        return Some(port_index as u16 + range.0);
                    }
                    port_index = port_index - ports_in_range;
                }
                panic!("Port selection algorithm is broken")
            }
            Constraint::Only(port) => {
                if data
                    .port_ranges
                    .iter()
                    .any(|range| (range.0 <= port && port <= range.1))
                {
                    Some(port)
                } else {
                    None
                }
            }
        }
    }

    /// Try to read the relays, first from cache and if that fails from the resources.
    fn read_cached_relays(cache_path: &Path, resource_path: &Path) -> Result<ParsedRelays> {
        match ParsedRelays::from_file(cache_path).chain_err(|| "Unable to read relays from cache") {
            Ok(value) => Ok(value),
            Err(error) => {
                debug!("{}", error.display_chain());
                ParsedRelays::from_file(resource_path)
            }
        }
    }
}

type RelayListUpdaterHandle = mpsc::Sender<()>;

struct RelayListUpdater {
    rpc_client: RelayListProxy<HttpHandle>,
    cache_path: PathBuf,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    close_handle: mpsc::Receiver<()>,
}

impl RelayListUpdater {
    pub fn spawn(
        rpc_handle: HttpHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
    ) -> RelayListUpdaterHandle {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || Self::new(rpc_handle, cache_path, parsed_relays, rx).run());

        tx
    }

    fn new(
        rpc_handle: HttpHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
        close_handle: mpsc::Receiver<()>,
    ) -> Self {
        let rpc_client = RelayListProxy::new(rpc_handle);

        RelayListUpdater {
            rpc_client,
            cache_path,
            parsed_relays,
            close_handle,
        }
    }

    fn run(&mut self) {
        debug!("Starting relay list updater thread");
        while self.wait_for_next_iteration() {
            if self.should_update() {
                match self
                    .update()
                    .chain_err(|| "Failed to update list of relays")
                {
                    Ok(()) => info!("Updated list of relays"),
                    Err(error) => error!("{}", error.display_chain()),
                }
            }
        }
        debug!("Relay list updater thread has finished");
    }

    fn wait_for_next_iteration(&mut self) -> bool {
        use self::mpsc::RecvTimeoutError::*;

        match self.close_handle.recv_timeout(UPDATE_CHECK_INTERVAL) {
            Ok(()) => true,
            Err(Timeout) => true,
            Err(Disconnected) => false,
        }
    }

    fn should_update(&mut self) -> bool {
        match SystemTime::now().duration_since(self.lock_parsed_relays().last_updated()) {
            Ok(duration) => duration > UPDATE_INTERVAL,
            // If the clock is skewed we have no idea by how much or when the last update
            // actually was, better download again to get in sync and get a `last_updated`
            // timestamp corresponding to the new time.
            Err(_) => true,
        }
    }

    fn update(&mut self) -> Result<()> {
        let new_relay_list = self
            .download_relay_list()
            .chain_err(|| "Failed to download relay list")?;

        if let Err(error) = self.cache_relays(&new_relay_list) {
            let chained_error = error.chain_err(|| "Failed to update relay cache on disk");
            error!("{}", chained_error.display_chain());
        }

        let new_parsed_relays = ParsedRelays::from_relay_list(new_relay_list, SystemTime::now());
        info!(
            "Downloaded relay inventory has {} relays",
            new_parsed_relays.relays().len()
        );

        *self.lock_parsed_relays() = new_parsed_relays;

        Ok(())
    }

    fn download_relay_list(&mut self) -> Result<RelayList> {
        info!("Downloading list of relays...");

        let download_future = self
            .rpc_client
            .relay_list_v2()
            .map_err(|e| Error::with_chain(e, ErrorKind::DownloadError));
        let relay_list = Timer::default()
            .timeout(download_future, DOWNLOAD_TIMEOUT)
            .wait()?;

        Ok(relay_list)
    }

    /// Write a `RelayList` to the cache file.
    fn cache_relays(&self, relays: &RelayList) -> Result<()> {
        debug!("Writing relays cache to {}", self.cache_path.display());
        let file = File::create(&self.cache_path).chain_err(|| ErrorKind::RelayCacheError)?;
        serde_json::to_writer_pretty(io::BufWriter::new(file), relays)
            .chain_err(|| ErrorKind::SerializationError)
    }

    fn lock_parsed_relays(&self) -> MutexGuard<ParsedRelays> {
        self.parsed_relays
            .lock()
            .expect("A thread crashed while it held a lock to the list of relays")
    }
}
