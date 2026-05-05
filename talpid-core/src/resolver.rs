//! This module implements a forwarding DNS resolver with two states:
//! * In the `Blocked` state, most queries receive an empty response, but certain captive portal
//!   domains receive a spoofed answer. This fools the OS into thinking that it has connectivity.
//! * In the `Forwarding` state, queries are forwarded to a set of configured DNS servers. This
//!   lets us use the routing table to determine where to send them, instead of them being forced
//!   out on the primary interface (in some cases).
//!
//! See [start_resolver](crate::resolver::start_resolver).

use std::{
    io, iter,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, Weak},
    time::{Duration, Instant},
};

use futures::{
    SinkExt, StreamExt,
    channel::{mpsc, oneshot},
};

use hickory_proto::{
    op::{HeaderCounts, LowerQuery, Metadata},
    rr::{LowerName, RecordType},
};
use hickory_server::{
    Server,
    net::{DnsError, runtime::Time},
    proto::{
        op::{Header, MessageType, OpCode},
        rr::{RData, Record, domain::Name, rdata},
    },
    resolver::{
        //ResolveError, ResolveErrorKind, TokioResolver,
        TokioResolver,
        config::{NameServerConfig, ResolverConfig},
        lookup::Lookup,
        net::{NetError, runtime::TokioRuntimeProvider},
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    zone_handler::{
        AuthLookup, AuthLookupIter, MessageRequest, MessageResponse, MessageResponseBuilder,
        UpdateRequest,
    },
};
use rand::random_range;
use socket2::{Domain, Protocol, Socket, Type};
use std::sync::LazyLock;
use talpid_routing::data::RouteSocketMessage;
use talpid_types::drop_guard::{OnDrop, on_drop};
use tokio::{
    net::{self, UdpSocket},
    task::JoinHandle,
};

/// If a local DNS resolver should be used at all times.
///
/// This setting does not affect the error or blocked state. In those states, we will want to use
/// the local DNS resoler to work around Apple's captive portals check. Exactly how this is done is
/// documented elsewhere.
pub static LOCAL_DNS_RESOLVER: LazyLock<bool> = LazyLock::new(|| {
    let disable_local_dns_resolver = std::env::var("TALPID_DISABLE_LOCAL_DNS_RESOLVER")
        .map(|v| v != "0")
        // Use the local DNS resolver by default.
        .unwrap_or(false);

    if !disable_local_dns_resolver {
        log::debug!("Using local DNS resolver");
    }
    !disable_local_dns_resolver
});

/// Override the `filter_out_aaaa` flag, which prevents getaddrinfo from returning IPv6 addresses.
/// See [ResolverHandle::enable_forward] for more details.
static NEVER_FILTER_AAAA_QUERIES: LazyLock<bool> = LazyLock::new(|| {
    let never_filter_aaaa_queries = std::env::var("TALPID_NEVER_FILTER_AAAA_QUERIES")
        .map(|v| v != "0")
        // Disable this functionality
        .unwrap_or(false);

    if never_filter_aaaa_queries {
        log::debug!("Disabling filtering of AAAA queries");
    }
    never_filter_aaaa_queries
});

// Name of the loopback network device.
const LOOPBACK: &str = "lo0";

/// The port we should bind the local DNS resolver to.
const DNS_PORT: u16 = if cfg!(test) {
    1053 // use a value above 1000 to allow for running the tests without root privileges
} else {
    53
};

const ALLOWED_RECORD_TYPES: &[RecordType] = &[RecordType::A, RecordType::CNAME];
/// Note: These domains need to be fully qualified domain names.
/// - <https://en.wikipedia.org/wiki/Fully_qualified_domain_name>
/// - <https://github.com/hickory-dns/hickory-dns/issues/2932>
const CAPTIVE_PORTAL_DOMAINS: &[&str] = &["captive.apple.com.", "netcts.cdn-apple.com."];

static ALLOWED_DOMAINS: LazyLock<Vec<LowerName>> = LazyLock::new(|| {
    CAPTIVE_PORTAL_DOMAINS
        .iter()
        .map(|domain| LowerName::from(Name::from_str(domain).unwrap()))
        .collect()
});

const TTL_SECONDS: u32 = 3;
/// An IP address to be used in the DNS response to the captive domain query. The address itself
/// belongs to the documentation range so should never be reachable.
const RESOLVED_ADDR: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 1);

#[derive(Clone, Debug, PartialEq)]
pub struct LocalResolverConfig {
    /// Try to bind to a random address in the `127/8` subnet.
    pub use_random_loopback: bool,
}

impl Default for LocalResolverConfig {
    fn default() -> Self {
        Self {
            use_random_loopback: true,
        }
    }
}

/// Starts a resolver. Returns a cloneable handle, which can activate, deactivate and shut down the
/// resolver. When all instances of a handle are dropped, the server will stop.
pub async fn start_resolver(config: LocalResolverConfig) -> Result<ResolverHandle, Error> {
    let (resolver, resolver_handle) = LocalResolver::new(config).await?;
    tokio::spawn(resolver.run());
    Ok(resolver_handle)
}

/// Resolver errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to bind UDP socket
    #[error("Failed to bind UDP socket")]
    UdpBind,

    /// Failed to get local address of a bound UDP socket
    #[error("Failed to get local address of a bound UDP socket")]
    GetSocketAddr(#[source] io::Error),
}

/// A DNS resolver that forwards queries to some other DNS server
///
/// Is controlled by commands sent through [ResolverHandle]s.
/// When all [ResolverHandle]s are dropped, [Self::rx] will close and [Self::run] will exit.
struct LocalResolver {
    rx: mpsc::UnboundedReceiver<ResolverMessage>,
    dns_server_task: JoinHandle<()>,
    inner_resolver: Resolver,
    /// Which IP+port the local resolver is bound to.
    bound_to: SocketAddr,
}

/// A message to [LocalResolver]
enum ResolverMessage {
    /// Set resolver config
    SetConfig {
        /// New DNS config to use
        new_config: Config,
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },

    /// Send a DNS query to the resolver
    Query {
        dns_query: LowerQuery,

        /// Channel for the query response
        response_tx: oneshot::Sender<Result<Box<AuthLookup>, NetError>>,
    },

    /// Gracefully stop resolver
    Stop {
        /// Channel for the query response
        response_tx: oneshot::Sender<()>,
    },
}

/// Configuration for [Resolver]
#[derive(Debug, Default, Clone)]
enum Config {
    /// Drop DNS queries. For captive portal domains, return faux records.
    #[default]
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding {
        /// Remote DNS server to use
        dns_servers: Vec<IpAddr>,
        /// Whether to give an empty response to AAAA queries
        filter_out_aaaa: bool,
    },
}

enum Resolver {
    /// Drop DNS queries. For captive portal domains, return faux records
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding {
        resolver: Box<TokioResolver>,
        filter_out_aaaa: bool,
    },
}

impl Resolver {
    pub fn resolve(
        &self,
        query: LowerQuery,
        tx: oneshot::Sender<Result<Box<AuthLookup>, NetError>>,
    ) {
        match self {
            Resolver::Blocking => {
                let _ = tx.send(Self::resolve_blocked(query).map(Box::new));
            }
            Resolver::Forwarding {
                resolver,
                filter_out_aaaa,
            } => {
                let resolver = resolver.clone();
                let filter_out_aaaa = *filter_out_aaaa && !*NEVER_FILTER_AAAA_QUERIES;
                tokio::spawn(async move {
                    let lookup = Self::resolve_forward(*resolver, query, filter_out_aaaa)
                        .await
                        .map(Box::new);
                    let _ = tx.send(lookup);
                });
            }
        };
    }

    /// Resolution in blocked state will return spoofed records for captive portal domains.
    fn resolve_blocked(query: LowerQuery) -> Result<AuthLookup, NetError> {
        if !Self::is_captive_portal_domain(&query) {
            return Ok(AuthLookup::Empty);
        }

        let return_query = query.original().clone();
        let mut return_record = Record::update0(
            return_query.name().clone(),
            TTL_SECONDS,
            return_query.query_type(),
        );
        return_record.data = RData::A(rdata::A(RESOLVED_ADDR));

        log::trace!(
            "Spoofing query for captive portal domain: {}",
            return_query.name()
        );

        let lookup = Lookup::new_with_deadline(
            return_query,
            vec![return_record],
            Instant::now() + Duration::from_secs(3),
        );
        Ok(AuthLookup::Resolved(lookup))
    }

    /// Determines whether a DNS query is allowable. Currently, this implies that the query is
    /// either a `A` or a `CNAME` query for `captive.apple.com`.
    fn is_captive_portal_domain(query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && ALLOWED_DOMAINS.contains(query.name())
    }

    /// Forward DNS queries to the specified DNS resolver.
    async fn resolve_forward(
        resolver: TokioResolver,
        query: LowerQuery,
        filter_out_aaaa: bool,
    ) -> Result<AuthLookup, NetError> {
        let return_query = query.original().clone();

        if filter_out_aaaa && query.query_type() == RecordType::AAAA {
            log::trace!("Giving empty response to AAAA query");
            return Ok(AuthLookup::Empty);
        }

        let lookup = resolver
            .lookup(return_query.name().clone(), return_query.query_type())
            .await?;

        Ok(AuthLookup::Resolved(lookup))
    }
}

/// A handle to control a DNS resolver.
///
/// When all resolver handles are dropped, the resolver will stop.
#[derive(Clone, Debug)]
pub struct ResolverHandle {
    tx: Arc<mpsc::UnboundedSender<ResolverMessage>>,
    listening_addr: SocketAddr,
}

impl ResolverHandle {
    fn new(tx: Arc<mpsc::UnboundedSender<ResolverMessage>>, listening_addr: SocketAddr) -> Self {
        Self { tx, listening_addr }
    }

    /// Get socket address associated with the running DNS resolver.
    pub fn listening_addr(&self) -> SocketAddr {
        self.listening_addr
    }

    /// Set the DNS server to forward queries to `dns_servers`
    ///
    /// # Arguments
    ///
    /// `filter_out_aaaa`: This causes the resolver to always return empty responses for AAAA (IPv6)
    ///                    queries. This is useful on macOS when the primary interface has IPv6
    ///                    connectivity, but the VPN tunnel does not. When this is true, and the VPN
    ///                    tunnel lacks IPv6 connectivity, programs like Firefox will resolve IPv6
    ///                    addresses and may attempt to connect to them anyway (but fail).
    pub async fn enable_forward(&self, dns_servers: Vec<IpAddr>, filter_out_aaaa: bool) {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = self.tx.unbounded_send(ResolverMessage::SetConfig {
            new_config: Config::Forwarding {
                dns_servers,
                filter_out_aaaa,
            },
            response_tx,
        });

        let _ = response_rx.await;
    }

    // Disable forwarding
    pub async fn disable_forward(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = self.tx.unbounded_send(ResolverMessage::SetConfig {
            new_config: Config::Blocking,
            response_tx,
        });

        let _ = response_rx.await;
    }

    /// Gracefully shut down resolver
    pub async fn stop(self) {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = self
            .tx
            .unbounded_send(ResolverMessage::Stop { response_tx });
        let _ = response_rx.await;
    }
}

impl LocalResolver {
    /// Constructs a new filtering resolver and it's handle.
    async fn new(config: LocalResolverConfig) -> Result<(Self, ResolverHandle), Error> {
        let (command_tx, command_rx) = mpsc::unbounded();
        let command_tx = Arc::new(command_tx);
        let weak_tx = Arc::downgrade(&command_tx);

        let (socket, cleanup_ifconfig) = Self::new_random_socket(&config).await?;
        let resolver_addr = socket.local_addr().map_err(Error::GetSocketAddr)?;
        let mut server = Self::new_server(socket, weak_tx.clone())?;

        let dns_server_task = tokio::spawn(async move {
            // This drop guard will clean up the loopback IP addr alias when the task exits.
            let _cleanup_ifconfig = cleanup_ifconfig;

            log::info!("Running DNS resolver on {resolver_addr}");

            loop {
                let Err(err) = server.block_until_done().await else {
                    break; // Graceful shutdown
                };

                log::error!("DNS server unexpectedly stopped: {}", err);
                drop(server); // drop the old server since we need to create a new one

                // Exit if `command_tx` has been dropped.
                if weak_tx.strong_count() == 0 {
                    break;
                }

                log::debug!("Attempting to restart server");

                let socket = match net::UdpSocket::bind(resolver_addr).await {
                    Ok(socket) => socket,
                    Err(e) => {
                        log::error!("Failed to bind DNS server to {resolver_addr}: {e}");
                        break;
                    }
                };

                match Self::new_server(socket, weak_tx.clone()) {
                    Ok(new_server) => server = new_server,
                    Err(error) => {
                        log::error!("Failed to restart DNS server: {error}");
                        break;
                    }
                }
            }
        });

        let resolver = Self {
            rx: command_rx,
            dns_server_task,
            bound_to: resolver_addr,
            inner_resolver: Resolver::Blocking,
        };

        Ok((resolver, ResolverHandle::new(command_tx, resolver_addr)))
    }

    fn new_server(
        socket: UdpSocket,
        command_tx: Weak<mpsc::UnboundedSender<ResolverMessage>>,
    ) -> Result<Server<ResolverImpl>, Error> {
        let mut server = Server::new(ResolverImpl { tx: command_tx });

        server.register_socket(socket);

        Ok(server)
    }

    /// Create a new [net::UdpSocket] bound to port 53 on loopback.
    ///
    /// This socket will try to bind to the following IPs in sequential order:
    /// - random ip in the range 127.1-255.0-255.1-254 : 53
    /// - random ip in the range 127.1-255.0-255.1-254 : 53
    /// - random ip in the range 127.1-255.0-255.1-254 : 53
    /// - 127.0.0.1 : 53
    ///
    /// We do this to try and avoid collisions with other DNS servers running on the same system.
    ///
    /// If [LocalResolverConfig::use_random_loopback] is `false`, we will only try to bind to
    /// `127.0.0.1`.
    ///
    /// # Returns
    /// - The first successfully bound [UdpSocket]
    /// - An [OnDrop] guard that will delete the IP aliases added, if any.
    ///   If the guard is dropped while the socket is in use, calls to read/write will likely fail.
    async fn new_random_socket(config: &LocalResolverConfig) -> Result<(UdpSocket, OnDrop), Error> {
        use std::net::Ipv4Addr;

        let random_loopback = || async move {
            let addr = Ipv4Addr::new(
                127,
                random_range(1..=255),
                random_range(0..=255),
                random_range(1..=254),
            );

            // TODO: this command requires root privileges and will thus not work in `cargo test`.
            // This means that the tests will fall back to 127.0.0.1, and will not assert that the
            // ifconfig stuff actually works. We probably do want to test this, so what do?
            talpid_macos::net::add_alias(LOOPBACK, IpAddr::from(addr))
                .await
                .inspect_err(|e| {
                    log::warn!("Failed to add loopback {LOOPBACK} alias {addr}: {e}");
                })
                .ok()?;

            log::debug!("Created loopback address {addr}");

            let detect_removed_alias_task =
                tokio::spawn(detect_loopback_address_removal(IpAddr::from(addr)));

            // Clean up ip address when stopping the resolver
            let cleanup_ifconfig = on_drop(move || {
                tokio::task::spawn(async move {
                    detect_removed_alias_task.abort();

                    log::debug!("Cleaning up loopback address {addr}");
                    if let Err(e) =
                        talpid_macos::net::remove_alias(LOOPBACK, IpAddr::from(addr)).await
                    {
                        log::warn!("Failed to clean up {LOOPBACK} alias {addr}: {e}");
                    }
                });
            })
            .boxed();

            Some((addr, cleanup_ifconfig))
        };

        for attempt in 0.. {
            let (socket_addr, on_drop) = match attempt {
                ..3 if !config.use_random_loopback => continue,
                ..3 => match random_loopback().await {
                    Some(random) => random,
                    None => continue,
                },

                3 => (Ipv4Addr::LOCALHOST, OnDrop::noop()),
                4.. => break,
            };

            let sock = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
                Ok(sock) => sock,
                Err(error) => {
                    log::error!("Failed to open IPv4/UDP socket: {error}");
                    continue;
                }
            };

            // SO_NONBLOCK is required for turning this into a tokio socket.
            if let Err(error) = sock.set_nonblocking(true) {
                log::warn!("Failed to set socket as nonblocking: {error}");
                continue;
            }

            // SO_REUSEADDR allows us to bind to `127.x.y.z` even if another socket is bound to
            // `0.0.0.0`. This can happen e.g. when macOS "Internet Sharing" is turned on.
            if let Err(error) = sock.set_reuse_address(true) {
                log::warn!("Failed to set SO_REUSEADDR on resolver socket: {error}");
            }

            match sock.bind(&SocketAddr::from((socket_addr, DNS_PORT)).into()) {
                Ok(()) => {
                    let socket =
                        net::UdpSocket::from_std(sock.into()).expect("socket is non-blocking");
                    return Ok((socket, on_drop));
                }
                Err(err) => log::warn!("Failed to bind DNS server to {socket_addr}: {err}"),
            }
        }

        // See logs for details.
        Err(Error::UdpBind)
    }

    /// Runs the filtering resolver as an actor, listening for new queries instances.  When all
    /// related [ResolverHandle] instances are dropped, this function will return, closing the DNS
    /// server.
    async fn run(mut self) {
        let abort_handle = self.dns_server_task.abort_handle();
        let _abort_dns_server_task = on_drop(|| abort_handle.abort());
        let mut stop_tx = None;

        while let Some(request) = self.rx.next().await {
            match request {
                ResolverMessage::SetConfig {
                    new_config,
                    response_tx,
                } => {
                    log::trace!("Updating config: {new_config:?}");
                    if let Err(err) = self.update_config(new_config) {
                        log::warn!("Failed to update DNS resolver config: {err}");
                        continue;
                    };
                    flush_system_cache();
                    let _ = response_tx.send(());
                }
                ResolverMessage::Query {
                    dns_query,
                    response_tx,
                } => {
                    self.inner_resolver.resolve(dns_query, response_tx);
                }
                ResolverMessage::Stop { response_tx } => {
                    stop_tx = Some(response_tx);
                    break;
                }
            }
        }

        self.dns_server_task.abort();
        let _ = self.dns_server_task.await;

        if let Some(stop_tx) = stop_tx {
            let _ = stop_tx.send(());
        }
    }

    /// Update the current DNS config.
    fn update_config(&mut self, config: Config) -> Result<(), NetError> {
        match config {
            Config::Blocking => self.blocking(),
            Config::Forwarding {
                mut dns_servers,
                filter_out_aaaa,
            } => {
                // make sure not to accidentally forward queries to ourselves
                dns_servers.retain(|addr| *addr != self.bound_to.ip());
                self.forwarding(dns_servers, filter_out_aaaa)?;
            }
        };
        Ok(())
    }

    /// Turn into a blocking resolver.
    fn blocking(&mut self) {
        self.inner_resolver = Resolver::Blocking;
    }

    /// Turn into a forwarding resolver (forward DNS queries to `dns_servers`).
    fn forwarding(
        &mut self,
        dns_servers: Vec<IpAddr>,
        filter_out_aaaa: bool,
    ) -> Result<(), NetError> {
        let forward_server_config = dns_servers
            .into_iter()
            .map(NameServerConfig::udp_and_tcp)
            .collect();

        let forward_config = ResolverConfig::from_parts(None, vec![], forward_server_config);
        let resolver =
            TokioResolver::builder_with_config(forward_config, TokioRuntimeProvider::default())
                .build()?;

        self.inner_resolver = Resolver::Forwarding {
            resolver: Box::new(resolver),
            filter_out_aaaa,
        };
        Ok(())
    }
}

/// Flush the DNS cache.
fn flush_system_cache() {
    if let Err(error) = kill_mdnsresponder() {
        log::error!("Failed to kill mDNSResponder: {error}");
    }
}

const MDNS_RESPONDER_PATH: &str = "/usr/sbin/mDNSResponder";

/// Find and kill mDNSResponder. The OS will restart the service.
fn kill_mdnsresponder() -> io::Result<()> {
    if let Some(mdns_pid) = talpid_macos::process::pid_of_path(MDNS_RESPONDER_PATH) {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(mdns_pid),
            nix::sys::signal::SIGHUP,
        )?;
    }
    Ok(())
}

/// Detect when the loopback address is removed on the loopback interface, and add it back whenever
/// that occurs.
async fn detect_loopback_address_removal(addr: IpAddr) -> Result<(), talpid_routing::RouteError> {
    let mut routing_table = talpid_routing::RoutingTable::new().map_err(|e| {
        log::warn!("Failed to create routing table interface: {e}");
        e
    })?;

    // Listen for the loopback address being removed, and add it back if that happens
    loop {
        let Ok(msg) = routing_table.next_message().await else {
            log::trace!("Failed to read next message from routing table");
            continue;
        };

        let RouteSocketMessage::DeleteAddress(msg) = msg else {
            continue;
        };

        // The deleted address either matches the one we care about, or we do not know
        let matches_addr = msg
            .address()
            .map(|deleted_addr| deleted_addr == addr)
            .unwrap_or(true);
        if !matches_addr {
            continue;
        }

        // Sleep for a bit so we do not spin if something weird is going on
        tokio::time::sleep(Duration::from_secs(1)).await;

        log::debug!("Detected possible removal of loopback address {addr}. Adding it back");

        talpid_macos::net::add_alias(LOOPBACK, addr)
            .await
            .inspect_err(|e| {
                log::warn!("Failed to add loopback {LOOPBACK} alias {addr}: {e}");
            })
            .ok();
    }
}

/// An implementation of [hickory_server::server::RequestHandler] that forwards queries to
/// `FilteringResolver`.
struct ResolverImpl {
    tx: Weak<mpsc::UnboundedSender<ResolverMessage>>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a AuthLookup,
    ) -> LookupResponse<'a> {
        let metadata = Metadata::new(message.id(), MessageType::Response, OpCode::Query);
        MessageResponseBuilder::from_message_request(message).build(
            metadata,
            lookup.iter(),
            // forwarder responses only contain query answers, no ns,soa or additionals
            iter::empty(),
            iter::empty(),
            iter::empty(),
        )
    }

    /// This function is called when a DNS query is sent to the local resolver
    async fn lookup<R: ResponseHandler>(&self, request: &Request, mut response_handler: R) {
        let Some(tx_ref) = self.tx.upgrade() else {
            return;
        };

        let message: &MessageRequest = request;

        let mut tx = (*tx_ref).clone();
        // Flush all DNS queries
        for query in message.queries.queries() {
            let (response_tx, response_rx) = oneshot::channel();
            let _ = tx
                .send(ResolverMessage::Query {
                    dns_query: query.clone(),
                    response_tx,
                })
                .await;

            let Ok(lookup_result) = response_rx.await else {
                // cancelled
                return;
            };
            let response_result = match lookup_result {
                Ok(ref lookup) => {
                    let response = Self::build_response(message, lookup.as_ref());
                    response_handler.send_response(response).await
                }
                Err(NetError::Dns(DnsError::ResponseCode(response_code))) => {
                    let response = MessageResponseBuilder::from_message_request(message)
                        .error_msg(&message.metadata, response_code);
                    response_handler.send_response(response).await
                }
                Err(_) => {
                    let response = Self::build_response(message, &AuthLookup::Empty);
                    response_handler.send_response(response).await
                }
            };
            if let Err(err) = response_result {
                log::error!("Failed to send response: {err}");
            }
        }
    }
}

type LookupResponse<'a> = MessageResponse<
    'a,
    'a,
    AuthLookupIter<'a>,
    iter::Empty<&'a Record>,
    iter::Empty<&'a Record>,
    iter::Empty<&'a Record>,
>;

#[async_trait::async_trait]
impl RequestHandler for ResolverImpl {
    async fn handle_request<R: ResponseHandler, T: Time>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        let empty_header = || Header {
            metadata: Metadata::new(0, MessageType::Query, OpCode::Query),
            counts: HeaderCounts {
                queries: 0,
                answers: 0,
                authorities: 0,
                additionals: 0,
            },
        };
        if !request.src().ip().is_loopback() {
            log::error!("Dropping a stray request from outside: {}", request.src());
            return empty_header().into();
        }
        if let MessageType::Query = request.metadata.message_type {
            match request.metadata.op_code {
                OpCode::Query => {
                    self.lookup(request, response_handle).await;
                }
                _ => {
                    log::trace!("Dropping non-query request: {:?}", request);
                }
            };
        }

        return empty_header().into();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hickory_server::resolver::config::{NameServerConfig, ResolverConfig};
    use std::{net::UdpSocket, sync::Mutex, thread};
    use typed_builder::TypedBuilder;

    /// Can't have multiple local resolvers running at the same time, as they will try to bind to
    /// the same address and port. The tests below use this lock to run sequentially.
    static LOCK: Mutex<()> = Mutex::new(());

    async fn start_resolver() -> ResolverHandle {
        // NOTE: We're disabling lo0 aliases
        super::start_resolver(LocalResolverConfig {
            // Bind resolver to 127.0.0.1
            use_random_loopback: false,
        })
        .await
        .unwrap()
    }

    fn get_test_resolver(addr: SocketAddr) -> TokioResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            vec![NameServerConfig::udp_and_tcp(addr.ip())],
        );
        TokioResolver::builder_with_config(resolver_config, TokioRuntimeProvider::default())
            .build()
            .unwrap()
    }

    /// Test whether we can successfully bind the socket even if the address is already used to
    /// in different scenarios.
    ///
    /// # Note
    ///
    /// This test does not test aliases on lo0, as that requires root privileges.
    #[test_log::test]
    fn test_bind() {
        let _mutex = LOCK.lock().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // bind() succeeds if wildcard address is bound without REUSEADDR and REUSEPORT
            let _sock = bind_sock(
                BindParams::builder()
                    .bind_addr(format!("0.0.0.0:{DNS_PORT}").parse().unwrap())
                    .reuse_addr(false)
                    .reuse_port(false)
                    .build(),
            )
            .unwrap();

            let handle = start_resolver().await;
            let test_resolver = get_test_resolver(handle.listening_addr());
            test_resolver
                .ipv4_lookup(&ALLOWED_DOMAINS[0])
                .await
                .expect("lookup should succeed");
            drop(_sock);
            handle.stop().await;
            thread::sleep(Duration::from_millis(300));

            // bind() succeeds if wildcard address is bound with REUSEADDR and REUSEPORT
            let _sock = bind_sock(
                BindParams::builder()
                    .bind_addr(format!("0.0.0.0:{DNS_PORT}").parse().unwrap())
                    .reuse_addr(true)
                    .reuse_port(true)
                    .build(),
            )
            .unwrap();

            let handle = start_resolver().await;
            let test_resolver = get_test_resolver(handle.listening_addr());
            test_resolver
                .ipv4_lookup(&ALLOWED_DOMAINS[0])
                .await
                .expect("lookup should succeed");
            drop(_sock);
            handle.stop().await;

            // bind() should succeeds if 127.0.0.1 is already bound without REUSEADDR and REUSEPORT
            // NOTE: We cannot test this as creating an alias requires root privileges.
        });
    }

    #[test_log::test]
    fn test_successful_lookup() {
        let _mutex = LOCK.lock().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.block_on(start_resolver());
        let test_resolver = get_test_resolver(handle.listening_addr());

        rt.block_on(async move {
            for domain in &*ALLOWED_DOMAINS {
                test_resolver.lookup(domain, RecordType::A).await?;
            }
            Ok::<(), NetError>(())
        })
        .expect("Resolution of domains failed");
    }

    #[test_log::test]
    fn test_failed_lookup() {
        let _mutex = LOCK.lock().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.block_on(start_resolver());
        let test_resolver = get_test_resolver(handle.listening_addr());

        let captive_portal_domain = LowerName::from(Name::from_str("apple.com").unwrap());
        let resolver_result = rt.block_on(async move {
            test_resolver
                .lookup(captive_portal_domain, RecordType::A)
                .await
        });
        assert!(
            resolver_result.is_err(),
            "Non-whitelisted DNS request should fail"
        )
    }

    /// Test that we close the socket when shutting down the local resolver.
    #[test_log::test]
    fn test_unbind_socket_on_stop() {
        let _mutex = LOCK.lock().unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let config = LocalResolverConfig {
            // Bind resolver to 127.0.0.1 so that we can easily bind to the same address here.
            use_random_loopback: false,
        };
        let handle = rt.block_on(super::start_resolver(config)).unwrap();
        let addr = handle.listening_addr();
        assert_eq!(addr, SocketAddr::from((Ipv4Addr::LOCALHOST, DNS_PORT)));
        rt.block_on(handle.stop());
        thread::sleep(Duration::from_millis(300));
        UdpSocket::bind(addr).expect("Failed to bind to a port that should have been removed");
    }

    #[derive(TypedBuilder)]
    struct BindParams {
        bind_addr: SocketAddr,
        reuse_addr: bool,
        reuse_port: bool,
        #[builder(default)]
        connect_addr: Option<SocketAddr>,
    }

    /// Helper function for creating and binding a UDP socket
    fn bind_sock(params: BindParams) -> io::Result<UdpSocket> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

        let addr = params.bind_addr;
        sock.set_reuse_address(params.reuse_addr)?;
        sock.set_reuse_port(params.reuse_port)?;
        sock.bind(&addr.into())?;

        if let Some(addr) = params.connect_addr {
            sock.connect(&addr.into())?;
        }

        println!(
            "Bound to {} (reuseport: {}, reuseaddr: {})",
            params.bind_addr, params.reuse_port, params.reuse_addr
        );
        Ok(sock.into())
    }
}
