use super::{config::TEST_CONFIG, Error, PING_TIMEOUT, WAIT_FOR_TUNNEL_STATE_TIMEOUT};
use crate::network_monitor::{
    self, start_packet_monitor, MonitorOptions, MonitorUnexpectedlyStopped, PacketMonitor,
};
use futures::StreamExt;
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{
    constraints::Constraint,
    location::Location,
    relay_constraints::{
        BridgeSettings, GeographicLocationConstraint, LocationConstraint, RelaySettings,
    },
    relay_list::{Relay, RelayList},
    states::TunnelState,
};
use pnet_packet::ip::IpNextHeaderProtocols;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};
use talpid_types::net::wireguard::{PeerConfig, PrivateKey, TunnelConfig};
use test_rpc::{package::Package, AmIMullvad, ServiceClient};
use tokio::time::timeout;

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
) -> Result<ProbeResult, Error> {
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

    Ok(result)
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
    timeout(PING_TIMEOUT, rpc.send_ping(interface, dest))
        .await
        .map_err(|_| Error::PingTimeout)?
        .map_err(Error::Rpc)
}

/// Try to connect to a Mullvad Tunnel.
///
/// If that fails to begin to connect, [`Error::DaemonError`] is returned. If it fails to connect
/// after that, the daemon ends up in the [`TunnelState::Error`] state, and
/// [`Error::UnexpectedErrorState`] is returned.
pub async fn connect_and_wait(mullvad_client: &mut MullvadProxyClient) -> Result<(), Error> {
    log::info!("Connecting");

    mullvad_client
        .connect_tunnel()
        .await
        .map_err(|error| Error::Daemon(format!("failed to begin connecting: {}", error)))?;

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

    Ok(())
}

pub async fn disconnect_and_wait(mullvad_client: &mut MullvadProxyClient) -> Result<(), Error> {
    log::info!("Disconnecting");

    mullvad_client
        .disconnect_tunnel()
        .await
        .map_err(|error| Error::Daemon(format!("failed to begin disconnecting: {}", error)))?;
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
    relay_settings: RelaySettings,
) -> Result<(), Error> {
    mullvad_client
        .set_relay_settings(relay_settings)
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
        },
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::new(10, 64, 10, 1),
        ipv6_gateway: None,
        #[cfg(target_os = "linux")]
        fwmark: None,
    }
}

pub fn get_app_env() -> HashMap<String, String> {
    use mullvad_api::env;
    use std::net::ToSocketAddrs;

    let api_host = format!("api.{}", TEST_CONFIG.mullvad_host);
    let api_addr = format!("{api_host}:443")
        .to_socket_addrs()
        .expect("failed to resolve API host")
        .next()
        .unwrap();

    let api_host_env = (env::API_HOST_VAR.to_string(), api_host);
    let api_addr_env = (env::API_ADDR_VAR.to_string(), api_addr.to_string());

    [api_host_env, api_addr_env].into_iter().collect()
}

/// Return a filtered version of the daemon's relay list.
///
/// * `mullvad_client` - An interface to the Mullvad daemon.
/// * `critera` - A function used to determine which relays to return.
pub async fn filter_relays<Filter>(
    mullvad_client: &mut MullvadProxyClient,
    criteria: Filter,
) -> Result<Vec<Relay>, Error>
where
    Filter: Fn(&Relay) -> bool,
{
    let relay_list: RelayList = mullvad_client
        .get_relay_locations()
        .await
        .map_err(|error| Error::Daemon(format!("Failed to obtain relay list: {}", error)))?;

    Ok(relay_list
        .relays()
        .filter(|relay| criteria(relay))
        .cloned()
        .collect())
}

/// Convenience function for constructing a constraint from a given [`Relay`].
///
/// # Panics
///
/// The relay must have a location set.
pub fn into_constraint(relay: &Relay) -> Constraint<LocationConstraint> {
    relay
        .location
        .as_ref()
        .map(
            |Location {
                 country_code,
                 city_code,
                 ..
             }| {
                GeographicLocationConstraint::Hostname(
                    country_code.to_string(),
                    city_code.to_string(),
                    relay.hostname.to_string(),
                )
            },
        )
        .map(LocationConstraint::Location)
        .map(Constraint::Only)
        .expect("relay is missing location")
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
                // NOTE: Many packets will likely be observed for API traffic. Rather than filtering all
                // of those specifically, simply fail if our probes are observed.
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
