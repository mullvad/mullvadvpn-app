use chrono::{DateTime, Local};
use error_chain::ChainedError;
use futures::Future;

use mullvad_rpc::{HttpHandle, RelayListProxy};
use mullvad_types::location::Location;
use mullvad_types::relay_constraints::{
    Constraint, LocationConstraint, Match, OpenVpnConstraints, RelayConstraints, TunnelConstraints,
};
use mullvad_types::relay_list::{Relay, RelayList, RelayTunnels};

use serde_json;

use talpid_types::net::{TransportProtocol, TunnelEndpoint, TunnelEndpointData};

use std::fs::File;
use std::net::IpAddr;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::time::{self, Duration, Instant, SystemTime};
use std::{io, thread};

use rand::{self, Rng, ThreadRng};
use tokio_timer::{timer, DeadlineError, Timer};

const RELAYS_FILENAME: &str = "relays.json";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);
const MAX_CACHE_AGE: Duration = Duration::from_secs(60 * 60 * 24);

error_chain! {
    errors {
        RelayCacheError { description("Error with relay cache on disk") }
        DownloadError { description("Error when trying to download the list of relays") }
        DownloadTimeoutError { description("Timed out when trying to download the list of relays") }
        NoRelay { description("No relays matching current constraints") }
        SerializationError { description("Error in serialization of relaylist") }
    }
}

impl From<DeadlineError<Error>> for Error {
    fn from(e: DeadlineError<Error>) -> Error {
        match e.into_inner() {
            Some(inner_e) => inner_e,
            None => Error::from_kind(ErrorKind::DownloadTimeoutError),
        }
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
    _updater: RelayListUpdaterHandle,
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
                .format(::logging::DATE_TIME_FORMAT_STR)
        );
        let parsed_relays = Arc::new(Mutex::new(unsynchronized_parsed_relays));
        let updater = RelayListUpdater::spawn(rpc_handle, cache_path, parsed_relays.clone());
        RelaySelector {
            parsed_relays,
            rng: rand::thread_rng(),
            _updater: updater,
        }
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
            let mut i: u64 = self.rng.gen_range(0, total_weight + 1);
            Some(
                relays
                    .iter()
                    .find(|relay| {
                        i = i.saturating_sub(relay.weight);
                        i == 0
                    }).unwrap(),
            )
        }
    }

    fn get_random_tunnel(&mut self, tunnels: &RelayTunnels) -> Option<TunnelEndpointData> {
        self.rng
            .choose(&tunnels.openvpn)
            .cloned()
            .map(|openvpn_endpoint| TunnelEndpointData::OpenVpn(openvpn_endpoint))
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
    timer: TimerHandle,
}

impl RelayListUpdater {
    pub fn spawn(
        rpc_handle: HttpHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
    ) -> RelayListUpdaterHandle {
        let timer = Self::start_timer();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || Self::new(rpc_handle, cache_path, parsed_relays, rx, timer).run());

        tx
    }

    fn start_timer() -> TimerHandle {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut timer = Timer::default();
            let alive = Arc::new(AtomicBool::new(true));

            let _ = tx.send(TimerHandle {
                handle: timer.handle(),
                alive: alive.clone(),
            });

            while alive.load(Ordering::Relaxed) {
                timer.turn(None).expect("Timer failed to run iteration");
            }
        });

        rx.recv().expect("Failed to create timer")
    }

    fn new(
        rpc_handle: HttpHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
        close_handle: mpsc::Receiver<()>,
        timer: TimerHandle,
    ) -> Self {
        let rpc_client = RelayListProxy::new(rpc_handle);

        RelayListUpdater {
            rpc_client,
            cache_path,
            parsed_relays,
            close_handle,
            timer,
        }
    }

    fn run(&mut self) {
        debug!("Starting relay list updater thread");
        while self.wait_for_next_iteration() {
            trace!("Relay list updater iteration");
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

        match self.close_handle.recv_timeout(UPDATE_INTERVAL) {
            Ok(()) => true,
            Err(Timeout) => true,
            Err(Disconnected) => false,
        }
    }

    fn should_update(&mut self) -> bool {
        match SystemTime::now().duration_since(self.lock_parsed_relays().last_updated()) {
            Ok(duration) => duration > MAX_CACHE_AGE,
            Err(_) => false,
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

        let timeout_instant = Instant::now() + DOWNLOAD_TIMEOUT;
        let download_future = self
            .rpc_client
            .relay_list()
            .map_err(|e| Error::with_chain(e, ErrorKind::DownloadError));
        let relay_list = self
            .timer
            .deadline(download_future, timeout_instant)
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

struct TimerHandle {
    handle: timer::Handle,
    alive: Arc<AtomicBool>,
}

impl Deref for TimerHandle {
    type Target = timer::Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
    }
}
