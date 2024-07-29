use super::{config::TEST_CONFIG, Error, TestContext, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
use crate::network_monitor::{
    self, start_packet_monitor, MonitorOptions, MonitorUnexpectedlyStopped, PacketMonitor,
};
use anyhow::{anyhow, bail, ensure, Context};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_relay_selector::{
    query::RelayQuery, GetRelay, RelaySelector, SelectorConfig, WireguardConfig,
};
use mullvad_types::{
    constraints::Constraint,
    location::Location,
    relay_constraints::{
        BridgeSettings, GeographicLocationConstraint, LocationConstraint, RelayConstraints,
        RelaySettings,
    },
    relay_list::Relay,
    states::TunnelState,
};
use pcap::Direction;
use pnet_packet::ip::IpNextHeaderProtocols;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};
use talpid_types::net::wireguard::{PeerConfig, PrivateKey, TunnelConfig};
use test_rpc::{meta::Os, package::Package, AmIMullvad, ServiceClient, SpawnOpts};
use tokio::time::sleep;

pub const THROTTLE_RETRY_DELAY: Duration = Duration::from_secs(120);

const CHECKER_FILENAME_WINDOWS: &str = "connection-checker.exe";
const CHECKER_FILENAME_UNIX: &str = "connection-checker";

const AM_I_MULLVAD_TIMEOUT_MS: u64 = 10000;
const LEAK_TIMEOUT_MS: u64 = 500;

/// Timeout of [ConnCheckerHandle::check_connection].
const CONN_CHECKER_TIMEOUT: Duration = Duration::from_millis(
    AM_I_MULLVAD_TIMEOUT_MS // https://am.i.mullvad.net timeout
    + LEAK_TIMEOUT_MS // leak-tcp timeout
    + LEAK_TIMEOUT_MS // leak-icmp timeout
    + 1000, // plus some extra grace time
);

#[macro_export]
macro_rules! assert_tunnel_state {
    ($mullvad_client:expr, $pattern:pat) => {{
        let state = $mullvad_client.get_tunnel_state().await?;
        assert!(matches!(state, $pattern), "state: {:?}", state);
    }};
}

pub fn get_package_desc(name: &str) -> Result<Package, Error> {
    Ok(Package {
        path: Path::new(&TEST_CONFIG.artifacts_dir).join(name),
    })
}

/// Reboot the guest virtual machine.
///
/// # macOS
/// The tunnel must be reconfigured after the virtual machine is up,
/// or macOS refuses to assign an IP. The reasons for this are poorly understood.
pub async fn reboot(rpc: &mut ServiceClient) -> Result<(), Error> {
    rpc.reboot().await?;

    #[cfg(target_os = "macos")]
    crate::vm::network::macos::configure_tunnel()
        .await
        .map_err(|error| Error::Other(format!("Failed to recreate custom wg tun: {error}")))?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct ProbeResult {
    tcp: usize,
    udp: usize,
    icmp: usize,
}

impl ProbeResult {
    pub fn all(&self) -> bool {
        self.tcp > 0 && self.udp > 0 && self.icmp > 0
    }

    pub fn none(&self) -> bool {
        !self.any()
    }

    pub fn any(&self) -> bool {
        self.tcp > 0 || self.udp > 0 || self.icmp > 0
    }
}

/// Return whether the guest exit IP is a Mullvad relay
pub async fn using_mullvad_exit(rpc: &ServiceClient) -> bool {
    log::info!("Test whether exit IP is a mullvad relay");
    geoip_lookup_with_retries(rpc)
        .await
        .unwrap()
        .mullvad_exit_ip
}

/// Get VPN tunnel interface name
pub async fn get_tunnel_interface(client: &mut MullvadProxyClient) -> Option<String> {
    match client.get_tunnel_state().await.ok()? {
        TunnelState::Connecting { endpoint, .. } | TunnelState::Connected { endpoint, .. } => {
            endpoint.tunnel_interface
        }
        _ => None,
    }
}

/// Sends a number of probes and returns the number of observed packets (UDP, TCP, or ICMP)
pub async fn send_guest_probes(
    rpc: ServiceClient,
    interface: String,
    destination: SocketAddr,
) -> ProbeResult {
    const MONITOR_DURATION: Duration = Duration::from_secs(8);

    let pktmon = start_packet_monitor(
        move |packet| packet.destination.ip() == destination.ip(),
        MonitorOptions {
            direction: Some(network_monitor::Direction::In),
            timeout: Some(MONITOR_DURATION),
            ..Default::default()
        },
    )
    .await;

    let send_handle = tokio::spawn(send_guest_probes_without_monitor(
        rpc,
        Some(interface),
        destination,
    ));

    let monitor_result = pktmon.wait().await.unwrap();

    send_handle.abort();
    let _ = send_handle.await;

    let mut result = ProbeResult::default();

    for pkt in monitor_result.packets {
        match pkt.protocol {
            IpNextHeaderProtocols::Tcp => {
                result.tcp = result.tcp.saturating_add(1);
            }
            IpNextHeaderProtocols::Udp => {
                result.udp = result.udp.saturating_add(1);
            }
            IpNextHeaderProtocols::Icmp => {
                result.icmp = result.icmp.saturating_add(1);
            }
            _ => (),
        }
    }

    result
}

/// Send one probe per transport protocol to `destination` without running a packet monitor
pub async fn send_guest_probes_without_monitor(
    rpc: ServiceClient,
    interface: Option<String>,
    destination: SocketAddr,
) {
    let bind_addr = if let Some(ref interface) = interface {
        SocketAddr::new(
            rpc.get_interface_ip(interface.clone())
                .await
                .expect("failed to obtain interface IP"),
            0,
        )
    } else {
        "0.0.0.0:0".parse().unwrap()
    };

    let tcp_rpc = rpc.clone();
    let tcp_interface = interface.clone();
    let tcp_send = async move {
        tcp_rpc
            .send_tcp(tcp_interface, bind_addr, destination)
            .await
    };
    let udp_rpc = rpc.clone();
    let udp_interface = interface.clone();
    let udp_send = async move {
        udp_rpc
            .send_udp(udp_interface, bind_addr, destination)
            .await
    };
    let icmp = async move { ping_with_timeout(&rpc, destination.ip(), interface).await };
    let _ = tokio::join!(tcp_send, udp_send, icmp);
}

pub async fn ping_with_timeout(
    rpc: &ServiceClient,
    dest: IpAddr,
    interface: Option<String>,
) -> Result<(), Error> {
    const DEFAULT_PING_SIZE: usize = 64;

    rpc.send_ping(dest, interface, DEFAULT_PING_SIZE)
        .await
        .map_err(Error::Rpc)
}

pub async fn ping_sized_with_timeout(
    rpc: &ServiceClient,
    dest: IpAddr,
    interface: Option<String>,
    size: usize,
) -> Result<(), Error> {
    rpc.send_ping(dest, interface, size)
        .await
        .map_err(Error::Rpc)
}

/// Return the first address that `host` resolves to
pub async fn resolve_hostname_with_retries(
    host: impl tokio::net::ToSocketAddrs,
) -> std::io::Result<SocketAddr> {
    const MAX_ATTEMPTS: usize = 10;
    const RETRY_DELAY: Duration = Duration::from_secs(5);

    let mut last_error = None;
    let mut attempt = 0;

    loop {
        if attempt >= MAX_ATTEMPTS {
            break Err(last_error.unwrap_or(std::io::Error::other("lookup timed out")));
        }
        attempt += 1;

        match tokio::net::lookup_host(&host).await {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    // done
                    break Ok(addr);
                }
            }
            Err(err) => last_error = Some(err),
        }

        tokio::time::sleep(RETRY_DELAY).await;
    }
}

/// Get the mac address (if any) of a network interface (on the test-manager machine).
#[cfg(target_os = "linux")] // not used on macos
pub fn get_interface_mac(interface: &str) -> anyhow::Result<Option<[u8; 6]>> {
    let addrs = nix::ifaddrs::getifaddrs().map_err(|error| {
        log::error!("Failed to obtain interfaces: {}", error);
        test_rpc::Error::Syscall
    })?;

    let mut interface_exists = false;

    let mac_addr = addrs
        .filter(|addr| addr.interface_name == interface)
        .find_map(|addr| {
            // sadly, the only way of distinguishing between "iface doesn't exist" and
            // "iface has no mac addr" is to check if the interface appears anywhere in the list.
            interface_exists = true;

            let addr = addr.address.as_ref()?;
            let link_addr = addr.as_link_addr()?;
            let mac_addr = link_addr.addr()?;
            Some(mac_addr)
        });

    if interface_exists {
        Ok(mac_addr)
    } else {
        bail!("Interface not found: {interface:?}")
    }
}

/// Get the index of a network interface (on the test-manager machine).
#[cfg(target_os = "linux")] // not used on macos
pub fn get_interface_index(interface: &str) -> anyhow::Result<std::ffi::c_uint> {
    use nix::errno::Errno;
    use std::ffi::CString;

    let interface = CString::new(interface).context(anyhow!(
        "Failed to turn interface name {interface:?} into cstr"
    ))?;

    match unsafe { libc::if_nametoindex(interface.as_ptr()) } {
        0 => {
            let err = Errno::last();
            Err(err).context("Failed to get interface index")
        }
        i => Ok(i),
    }
}

/// Log in and retry if it fails due to throttling
pub async fn login_with_retries(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<(), mullvad_management_interface::Error> {
    loop {
        match mullvad_client
            .login_account(TEST_CONFIG.account_number.clone())
            .await
        {
            Err(mullvad_management_interface::Error::Rpc(status))
                if status.message().to_uppercase().contains("THROTTLED") =>
            {
                // Work around throttling errors by sleeping
                log::debug!(
                    "Login failed due to throttling. Sleeping for {} seconds",
                    THROTTLE_RETRY_DELAY.as_secs()
                );

                tokio::time::sleep(THROTTLE_RETRY_DELAY).await;
            }
            Err(err) => break Err(err),
            Ok(_) => break Ok(()),
        }
    }
}

/// Ensure that the test runner is logged in to an account.
///
/// This will first check whether we are logged in. If not, it will also try to login
/// on your behalf. If this function returns without any errors, we are logged in to a valid
/// account.
pub async fn ensure_logged_in(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<(), mullvad_management_interface::Error> {
    if mullvad_client.get_device().await?.is_logged_in() {
        return Ok(());
    }
    // We are apparently not logged in already.. Try to log in.
    login_with_retries(mullvad_client).await
}

/// Try to connect to a Mullvad Tunnel.
///
/// # Returns
/// - `Result::Ok(new_state)` if the daemon successfully connected to a tunnel
/// - `Result::Err` if:
///     - The daemon failed to even begin connecting. Then [`Error::Rpc`] is returned.
///     - The daemon started to connect but ended up in the [`TunnelState::Error`] state. Then
///       [`Error::UnexpectedErrorState`] is returned
pub async fn connect_and_wait(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<TunnelState, Error> {
    log::info!("Connecting");

    mullvad_client.connect_tunnel().await?;
    let new_state = wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connected { .. } | TunnelState::Error(..)
        )
    })
    .await?;

    if let TunnelState::Error(error_state) = new_state {
        return Err(Error::UnexpectedErrorState(error_state));
    }

    log::info!("Connected");

    Ok(new_state)
}

pub async fn disconnect_and_wait(mullvad_client: &mut MullvadProxyClient) -> Result<(), Error> {
    log::info!("Disconnecting");
    mullvad_client.disconnect_tunnel().await?;

    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(state, TunnelState::Disconnected { .. })
    })
    .await?;

    log::info!("Disconnected");

    Ok(())
}

pub async fn wait_for_tunnel_state(
    mut rpc: MullvadProxyClient,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    let events = rpc
        .events_listen()
        .await
        .map_err(|status| Error::Daemon(format!("Failed to get event stream: {}", status)))?;

    let state = rpc
        .get_tunnel_state()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to get tunnel state: {:?}", error)))?;

    if accept_state_fn(&state) {
        return Ok(state);
    }

    find_next_tunnel_state(events, accept_state_fn).await
}

pub async fn find_next_tunnel_state(
    stream: impl futures::Stream<Item = Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
    accept_state_fn: impl Fn(&mullvad_types::states::TunnelState) -> bool,
) -> Result<mullvad_types::states::TunnelState, Error> {
    tokio::time::timeout(
        WAIT_FOR_TUNNEL_STATE_TIMEOUT,
        find_daemon_event(stream, |daemon_event| match daemon_event {
            DaemonEvent::TunnelState(state) if accept_state_fn(&state) => Some(state),
            _ => None,
        }),
    )
    .await
    .map_err(|_error| Error::Daemon(String::from("Tunnel event listener timed out")))?
}

pub async fn find_daemon_event<Accept, AcceptedEvent>(
    mut event_stream: impl futures::Stream<Item = Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
    accept_event: Accept,
) -> Result<AcceptedEvent, Error>
where
    Accept: Fn(DaemonEvent) -> Option<AcceptedEvent>,
{
    loop {
        match event_stream.next().await {
            Some(Ok(daemon_event)) => match accept_event(daemon_event) {
                Some(accepted_event) => break Ok(accepted_event),
                None => continue,
            },
            Some(Err(status)) => {
                break Err(Error::Daemon(format!(
                    "Failed to get next event: {}",
                    status
                )))
            }
            None => break Err(Error::Daemon(String::from("Lost daemon event stream"))),
        }
    }
}

/// Set environment variables specified by `env` and restart the Mullvad daemon.
/// Returns a new [rpc client][`MullvadProxyClient`], since the old client *probably*
/// can't communicate with the new daemon.
///
/// # Note
/// This is just a thin wrapper around [`ServiceClient::set_daemon_environment`] which also
/// invalidates the old [`MullvadProxyClient`].
pub async fn restart_daemon_with<K, V, Env>(
    rpc: &ServiceClient,
    test_context: &TestContext,
    _: MullvadProxyClient, // Just consume the old proxy client
    env: Env,
) -> Result<MullvadProxyClient, Error>
where
    Env: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    rpc.set_daemon_environment(env).await?;
    // Need to create a new `mullvad_client` here after the restart
    // otherwise we *probably* can't communicate with the daemon.
    Ok(test_context.rpc_provider.new_client().await)
}

pub async fn geoip_lookup_with_retries(rpc: &ServiceClient) -> Result<AmIMullvad, Error> {
    const MAX_ATTEMPTS: usize = 5;
    const BEFORE_RETRY_DELAY: Duration = Duration::from_secs(2);

    let mut attempt = 0;

    loop {
        let result = rpc
            .geoip_lookup(TEST_CONFIG.mullvad_host.to_owned())
            .await
            .map_err(Error::GeoipLookup);

        attempt += 1;
        if result.is_ok() || attempt >= MAX_ATTEMPTS {
            return result;
        }

        tokio::time::sleep(BEFORE_RETRY_DELAY).await;
    }
}

pub struct AbortOnDrop<T>(Option<tokio::task::JoinHandle<T>>);

impl<T> AbortOnDrop<T> {
    pub fn new(inner: tokio::task::JoinHandle<T>) -> AbortOnDrop<T> {
        AbortOnDrop(Some(inner))
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

pub async fn set_relay_settings(
    mullvad_client: &mut MullvadProxyClient,
    relay_settings: impl Into<RelaySettings>,
) -> Result<(), Error> {
    mullvad_client
        .set_relay_settings(relay_settings.into())
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set relay settings: {}", error)))
}

pub async fn set_bridge_settings(
    mullvad_client: &mut MullvadProxyClient,
    bridge_settings: BridgeSettings,
) -> Result<(), Error> {
    mullvad_client
        .set_bridge_settings(bridge_settings)
        .await
        .map_err(|error| Error::Daemon(format!("Failed to set bridge settings: {}", error)))
}

/// Wait for the relay list to be updated, to make sure we have the overridden one.
/// Time out after a while.
pub async fn ensure_updated_relay_list(
    mullvad_client: &mut MullvadProxyClient,
) -> Result<(), mullvad_management_interface::Error> {
    let mut events = mullvad_client.events_listen().await?;
    mullvad_client.update_relay_locations().await?;

    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async move {
        while let Some(Ok(event)) = events.next().await {
            if matches!(event, DaemonEvent::RelayList(_)) {
                log::debug!("Received new relay list");
                break;
            }
        }
    })
    .await;

    Ok(())
}

pub fn unreachable_wireguard_tunnel() -> talpid_types::net::wireguard::ConnectionConfig {
    talpid_types::net::wireguard::ConnectionConfig {
        tunnel: TunnelConfig {
            private_key: PrivateKey::new_from_random(),
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 64, 10, 1))],
        },
        peer: PeerConfig {
            public_key: PrivateKey::new_from_random().public_key(),
            allowed_ips: vec![
                "0.0.0.0/0".parse().expect("Failed to parse ipv6 network"),
                "::0/0".parse().expect("Failed to parse ipv6 network"),
            ],
            endpoint: "1.3.3.7:1234".parse().unwrap(),
            psk: None,
            constant_packet_size: false,
        },
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 10, 1),
        ipv6_gateway: None,
        #[cfg(target_os = "linux")]
        fwmark: None,
    }
}

/// Return the current `MULLVAD_API_HOST` et al.
///
/// # Note
/// This is independent of the running daemon's environment.
/// It is solely dependant on the current value of [`TEST_CONFIG`].
pub async fn get_app_env() -> Result<HashMap<String, String>, Error> {
    use mullvad_api::env;

    let api_host = format!("api.{}", TEST_CONFIG.mullvad_host);
    let api_host_with_port = format!("{api_host}:443");
    let api_addr = resolve_hostname_with_retries(api_host_with_port)
        .await
        .map_err(Error::DnsLookup)?;

    Ok(HashMap::from_iter(vec![
        (env::API_HOST_VAR.to_string(), api_host),
        (env::API_ADDR_VAR.to_string(), api_addr.to_string()),
    ]))
}

/// Constrain the daemon to only select the relay selected with `query` when establishing all
/// future tunnels (until relay settings are updated, see [`set_relay_settings`]). Returns the
/// selected [`Relay`] for future reference.
///
/// # Note
/// This function does not handle bridges and multihop configurations (currently). There is no
/// particular reason for this other than it not being needed at the time, so feel free to extend
/// this function :).
pub async fn constrain_to_relay(
    mullvad_client: &mut MullvadProxyClient,
    query: RelayQuery,
) -> anyhow::Result<Relay> {
    /// Convert the result of invoking the relay selector to a relay constraint.
    fn convert_to_relay_constraints(
        selected_relay: GetRelay,
    ) -> anyhow::Result<(Relay, RelayConstraints)> {
        match selected_relay {
            GetRelay::Wireguard {
                inner: WireguardConfig::Singlehop { exit },
                ..
            }
            | GetRelay::OpenVpn { exit, .. } => {
                let location = into_constraint(&exit)?;
                let relay_constraints = RelayConstraints {
                    location,
                    ..Default::default()
                };
                Ok((exit, relay_constraints))
            }
            unsupported => bail!("Can not constrain to a {unsupported:?}"),
        }
    }

    // Construct a relay selector with up-to-date information from the runnin daemon's relay list
    let relay_list = mullvad_client.get_relay_locations().await?;
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
    // Select an(y) appropriate relay for the given query and constrain the daemon to only connect
    // to that specific relay (when connecting).
    let relay = relay_selector.get_relay_by_query(query)?;
    let (exit, relay_constraints) = convert_to_relay_constraints(relay)?;
    set_relay_settings(mullvad_client, RelaySettings::Normal(relay_constraints)).await?;

    Ok(exit)
}

/// Convenience function for constructing a constraint from a given [`Relay`].
///
/// # Panics
///
/// The relay must have a location set.
pub fn into_constraint(relay: &Relay) -> anyhow::Result<Constraint<LocationConstraint>> {
    relay
        .location
        .as_ref()
        .map(
            |Location {
                 country_code,
                 city_code,
                 ..
             }| {
                GeographicLocationConstraint::hostname(country_code, city_code, &relay.hostname)
            },
        )
        .map(LocationConstraint::Location)
        .map(Constraint::Only)
        .ok_or(anyhow!("relay is missing location"))
}

/// Ping monitoring made easy!
///
/// Continously ping some destination while monitoring to detect diverging
/// packets.
///
/// To customize [`Pinger`] before the pinging and network monitoring starts,
/// see [`PingerBuilder`]. Call [`start`](Pinger::start) to start pinging, and
/// [`stop`](Pinger::stop) to get the leak test results.
#[allow(dead_code)]
pub struct Pinger {
    // These values can be configured with [`PingerBuilder`].
    destination: SocketAddr,
    interval: tokio::time::Interval,
    // Run-time specific values
    pub guest_ip: IpAddr,
    ping_task: AbortOnDrop<tokio::task::JoinHandle<()>>,
    monitor: PacketMonitor,
}

impl Pinger {
    /// Create a [`Pinger`] with a default configuration.
    ///
    /// See [`PingerBuilder`] for details.
    pub async fn start(rpc: &test_rpc::ServiceClient) -> Pinger {
        let defaults = PingerBuilder::default();
        Self::start_with(defaults, rpc).await
    }

    /// Create a [`Pinger`] using the configuration of `builder`.
    ///
    /// See [`PingerBuilder`] for details on how to configure a [`Pinger`]
    /// before starting it.
    pub async fn start_with(builder: PingerBuilder, rpc: &test_rpc::ServiceClient) -> Pinger {
        // Get the associated IP address of the test runner on the default, non-tunnel interface.
        let guest_ip = obtain_guest_ip(rpc).await;
        log::debug!("Guest IP: {guest_ip}");

        // Start a network monitor
        log::debug!("Monitoring outgoing traffic");
        let monitor = start_packet_monitor(
            move |packet| {
                // NOTE: Many packets will likely be observed for API traffic. Rather than filtering
                // all of those specifically, simply fail if our probes are
                // observed.
                packet.source.ip() == guest_ip
                    && packet.destination.ip() == builder.destination.ip()
            },
            MonitorOptions::default(),
        )
        .await;

        // Start pinging
        //
        // Create some network activity for the network monitor to sniff.
        let ping_rpc = rpc.clone();
        let mut interval = tokio::time::interval(builder.interval.period());
        #[allow(clippy::async_yields_async)]
        let ping_task = AbortOnDrop::new(tokio::spawn(async move {
            loop {
                send_guest_probes_without_monitor(ping_rpc.clone(), None, builder.destination)
                    .await;
                interval.tick().await;
            }
        }));

        Pinger {
            destination: builder.destination,
            interval: builder.interval,
            guest_ip,
            ping_task,
            monitor,
        }
    }

    /// Stop pinging and extract the result of the network monitor.
    pub async fn stop(self) -> Result<network_monitor::MonitorResult, MonitorUnexpectedlyStopped> {
        // Abort the inner probe sender, which is accomplished by dropping the
        // join handle to the running task.
        drop(self.ping_task);
        self.monitor.into_result().await
    }

    /// Return the time period determining the cadence of pings that are sent.
    pub fn period(&self) -> tokio::time::Duration {
        self.interval.period()
    }
}

/// Returns the [`IpAddr`] of the default non-tunnel interface.
async fn obtain_guest_ip(rpc: &ServiceClient) -> IpAddr {
    let guest_iface = rpc
        .get_default_interface()
        .await
        .expect("failed to obtain default interface");
    rpc.get_interface_ip(guest_iface)
        .await
        .expect("failed to obtain non-tun IP")
}

/// Configure a [`Pinger`] before starting it.
pub struct PingerBuilder {
    destination: SocketAddr,
    interval: tokio::time::Interval,
}

#[allow(dead_code)]
impl PingerBuilder {
    /// Create a default [`PingerBuilder`].
    ///
    /// This is probably good enough for checking network traffic leaks when the
    /// test-runner is supposed to be blocked from sending or receiving *any*
    /// packets outside of localhost.
    pub fn default() -> PingerBuilder {
        PingerBuilder {
            destination: "1.1.1.1:1337".parse().unwrap(),
            interval: tokio::time::interval(Duration::from_secs(1)),
        }
    }

    /// Set the target to ping.
    pub fn destination(mut self, destination: SocketAddr) -> Self {
        self.destination = destination;
        self
    }

    /// How often a ping should be sent.
    pub fn interval(mut self, period: Duration) -> Self {
        self.interval = tokio::time::interval(period);
        self
    }
}

/// This helper spawns a seperate process which checks if we are connected to Mullvad, and tries to
/// leak traffic outside the tunnel by sending TCP, UDP, and ICMP packets to [LEAK_DESTINATION].
pub struct ConnChecker {
    rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
    leak_destination: SocketAddr,

    /// Path to the process binary.
    executable_path: String,

    /// Whether the process should be split when spawned. Needed on Linux.
    split: bool,

    /// Some arbitrary payload
    payload: Option<String>,
}

pub struct ConnCheckerHandle<'a> {
    checker: &'a mut ConnChecker,

    /// ID of the spawned process.
    pid: u32,
}

pub struct ConnectionStatus {
    /// True if <https://am.i.mullvad.net/> reported we are connected.
    am_i_mullvad: bool,

    /// True if we sniffed TCP packets going outside the tunnel.
    leaked_tcp: bool,

    /// True if we sniffed UDP packets going outside the tunnel.
    leaked_udp: bool,

    /// True if we sniffed ICMP packets going outside the tunnel.
    leaked_icmp: bool,
}

impl ConnChecker {
    pub fn new(
        rpc: ServiceClient,
        mullvad_client: MullvadProxyClient,
        leak_destination: SocketAddr,
    ) -> Self {
        let artifacts_dir = &TEST_CONFIG.artifacts_dir;
        let executable_path = match TEST_CONFIG.os {
            Os::Linux | Os::Macos => format!("{artifacts_dir}/{CHECKER_FILENAME_UNIX}"),
            Os::Windows => format!("{artifacts_dir}\\{CHECKER_FILENAME_WINDOWS}"),
        };

        Self {
            rpc,
            mullvad_client,
            leak_destination,
            split: false,
            executable_path,
            payload: None,
        }
    }

    /// Set a custom magic payload that the connection checker binary should use when leak-testing.
    pub fn payload(&mut self, payload: impl Into<String>) {
        self.payload = Some(payload.into())
    }

    /// Spawn the connecton checker process and return a handle to it.
    ///
    /// Dropping the handle will stop the process.
    /// **NOTE**: The handle must be dropped from a tokio runtime context.
    pub async fn spawn(&mut self) -> anyhow::Result<ConnCheckerHandle<'_>> {
        log::debug!("spawning connection checker");

        let opts = {
            let mut args = [
                "--interactive",
                "--timeout",
                &AM_I_MULLVAD_TIMEOUT_MS.to_string(),
                // try to leak traffic to LEAK_DESTINATION
                "--leak",
                &self.leak_destination.to_string(),
                "--leak-timeout",
                &LEAK_TIMEOUT_MS.to_string(),
                "--leak-tcp",
                "--leak-udp",
                "--leak-icmp",
            ]
            .map(String::from)
            .to_vec();

            if let Some(payload) = &self.payload {
                args.push("--payload".to_string());
                args.push(payload.clone());
            };

            SpawnOpts {
                attach_stdin: true,
                attach_stdout: true,
                args,
                ..SpawnOpts::new(&self.executable_path)
            }
        };

        let pid = self.rpc.spawn(opts).await?;

        if self.split && TEST_CONFIG.os == Os::Linux {
            self.mullvad_client
                .add_split_tunnel_process(pid as i32)
                .await?;
        }

        // TODO: The ST process monitor is a bit racy on macOS, such that processes aren't
        //       immediately recognized. This is a workaround until fixed.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        Ok(ConnCheckerHandle { pid, checker: self })
    }

    /// Enable split tunneling for the connection checker.
    pub async fn split(&mut self) -> anyhow::Result<()> {
        log::debug!("enable split tunnel");
        self.split = true;

        match TEST_CONFIG.os {
            Os::Linux => { /* linux programs can't be split until they are spawned */ }
            Os::Macos | Os::Windows => {
                self.mullvad_client
                    .add_split_tunnel_app(&self.executable_path)
                    .await?;
                self.mullvad_client.set_split_tunnel_state(true).await?;
            }
        }

        Ok(())
    }

    /// Disable split tunneling for the connection checker.
    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        log::debug!("disable split tunnel");
        self.split = false;

        match TEST_CONFIG.os {
            Os::Linux => {}
            Os::Macos | Os::Windows => {
                self.mullvad_client.set_split_tunnel_state(false).await?;
                self.mullvad_client
                    .remove_split_tunnel_app(&self.executable_path)
                    .await?;
            }
        }

        Ok(())
    }
}

impl ConnCheckerHandle<'_> {
    pub async fn split(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .add_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.split().await
    }

    pub async fn unsplit(&mut self) -> anyhow::Result<()> {
        if TEST_CONFIG.os == Os::Linux {
            self.checker
                .mullvad_client
                .remove_split_tunnel_process(self.pid as i32)
                .await?;
        }

        self.checker.unsplit().await
    }

    /// Assert that traffic is flowing through the Mullvad tunnel and that no packets are leaked.
    pub async fn assert_secure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is secure");
        let status = self.check_connection().await?;
        ensure!(status.am_i_mullvad);
        ensure!(!status.leaked_tcp);
        ensure!(!status.leaked_udp);
        ensure!(!status.leaked_icmp);

        Ok(())
    }

    /// Assert that traffic is NOT flowing through the Mullvad tunnel and that packets ARE leaked.
    pub async fn assert_insecure(&mut self) -> anyhow::Result<()> {
        log::info!("checking that connection is not secure");
        let status = self.check_connection().await?;
        ensure!(!status.am_i_mullvad);
        ensure!(status.leaked_tcp);
        ensure!(status.leaked_udp);
        ensure!(status.leaked_icmp);

        Ok(())
    }

    pub async fn check_connection(&mut self) -> anyhow::Result<ConnectionStatus> {
        // Monitor all pakets going to LEAK_DESTINATION during the check.
        let leak_destination = self.checker.leak_destination;
        let monitor = start_packet_monitor(
            move |packet| packet.destination.ip() == leak_destination.ip(),
            MonitorOptions {
                direction: Some(Direction::In),
                ..MonitorOptions::default()
            },
        )
        .await;

        // Write a newline to the connection checker to prompt it to perform the check.
        self.checker
            .rpc
            .write_child_stdin(self.pid, "Say the line, Bart!\r\n".into())
            .await?;

        // The checker responds when the check is complete.
        let line = self.read_stdout_line().await?;

        let monitor_result = monitor
            .into_result()
            .await
            .map_err(|_e| anyhow!("Packet monitor unexpectedly stopped"))?;

        Ok(ConnectionStatus {
            am_i_mullvad: parse_am_i_mullvad(line)?,

            leaked_tcp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Tcp),

            leaked_udp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Udp),

            leaked_icmp: (monitor_result.packets.iter())
                .any(|pkt| pkt.protocol == IpNextHeaderProtocols::Icmp),
        })
    }

    /// Try to a single line of output from the spawned process
    async fn read_stdout_line(&mut self) -> anyhow::Result<String> {
        // Add a timeout to avoid waiting forever.
        tokio::time::timeout(CONN_CHECKER_TIMEOUT, async {
            let mut line = String::new();

            // tarpc doesn't support streams, so we poll the checker process in a loop instead
            loop {
                let Some(output) = self.checker.rpc.read_child_stdout(self.pid).await? else {
                    bail!("got EOF from connection checker process");
                };

                if output.is_empty() {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                line.push_str(&output);

                if line.contains('\n') {
                    log::info!("output from child process: {output:?}");
                    return Ok(line);
                }
            }
        })
        .await
        .with_context(|| "Timeout reading stdout from connection checker")?
    }
}

impl Drop for ConnCheckerHandle<'_> {
    fn drop(&mut self) {
        let rpc = self.checker.rpc.clone();
        let pid = self.pid;

        let Ok(runtime_handle) = tokio::runtime::Handle::try_current() else {
            log::error!("ConnCheckerHandle dropped outside of a tokio runtime.");
            return;
        };

        runtime_handle.spawn(async move {
            // Make sure child process is stopped when this handle is dropped.
            // Closing stdin does the trick.
            let _ = rpc.close_child_stdin(pid).await;
        });
    }
}

/// Parse output from connection-checker. Returns true if connected to Mullvad.
fn parse_am_i_mullvad(result: String) -> anyhow::Result<bool> {
    Ok(if result.contains("You are connected") {
        true
    } else if result.contains("You are not connected") {
        false
    } else {
        bail!("Unexpected output from connection-checker: {result:?}")
    })
}
