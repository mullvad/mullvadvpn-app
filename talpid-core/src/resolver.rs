//! This module implements a forwarding DNS resolver with two states:
//! * In the `Blocked` state, most queries receive an empty response, but certain captive portal
//!   domains receive a spoofed answer. This fools the OS into thinking that it has connectivity.
//! * In the `Forwarding` state, queries are forwarded to a set of configured DNS servers. This
//!   lets us use the routing table to determine where to send them, instead of them being forced
//!   out on the primary interface (in some cases).
//!
//! See [start_resolver].
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, Weak},
    time::{Duration, Instant},
};

use futures::{
    channel::{mpsc, oneshot},
    future::Either,
    SinkExt, StreamExt,
};

use hickory_proto::{
    op::LowerQuery,
    rr::{LowerName, RecordType},
};
use hickory_server::{
    authority::{
        EmptyLookup, LookupObject, MessageRequest, MessageResponse, MessageResponseBuilder,
    },
    proto::{
        op::{header::MessageType, op_code::OpCode, Header},
        rr::{domain::Name, rdata, record_data::RData, Record},
    },
    resolver::{
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        error::{ResolveError, ResolveErrorKind},
        lookup::Lookup,
        TokioAsyncResolver,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    ServerFuture,
};
use rand::random;
use std::sync::LazyLock;
use talpid_types::drop_guard::{on_drop, OnDrop};
use tokio::{
    net::{self, UdpSocket},
    process::Command,
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

// Name of the loopback network device.
const LOOPBACK: &str = "lo0";

/// The port we should bind the local DNS resolver to.
const DNS_PORT: u16 = if cfg!(test) {
    1053 // use a value above 1000 to allow for running the tests without root privileges
} else {
    53
};

const ALLOWED_RECORD_TYPES: &[RecordType] = &[RecordType::A, RecordType::CNAME];
const CAPTIVE_PORTAL_DOMAINS: &[&str] = &["captive.apple.com", "netcts.cdn-apple.com"];

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

/// Starts a resolver. Returns a cloneable handle, which can activate, deactivate and shut down the
/// resolver. When all instances of a handle are dropped, the server will stop.
pub async fn start_resolver() -> Result<ResolverHandle, Error> {
    let (resolver, resolver_handle) = LocalResolver::new().await?;
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
        response_tx: oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
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
    },
}

enum Resolver {
    /// Drop DNS queries. For captive portal domains, return faux records
    Blocking,

    /// Forward DNS queries to a configured server
    Forwarding(TokioAsyncResolver),
}

impl Resolver {
    pub fn resolve(
        &self,
        query: LowerQuery,
        tx: oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
    ) {
        let lookup = match self {
            Resolver::Blocking => Either::Left(async move { Self::resolve_blocked(query) }),
            Resolver::Forwarding(resolver) => {
                Either::Right(Self::resolve_forward(resolver.clone(), query))
            }
        };

        tokio::spawn(async move {
            let _ = tx.send(lookup.await);
        });
    }

    /// Resolution in blocked state will return spoofed records for captive portal domains.
    fn resolve_blocked(
        query: LowerQuery,
    ) -> std::result::Result<Box<dyn LookupObject>, ResolveError> {
        log::info!("!!!");
        log::info!("!!! resolve_blocked: {query:?}");
        log::info!("!!!");

        if !Self::is_captive_portal_domain(&query) {
            return Ok(Box::new(EmptyLookup));
        }

        let return_query = query.original().clone();
        let mut return_record = Record::with(
            return_query.name().clone(),
            return_query.query_type(),
            TTL_SECONDS,
        );
        return_record.set_data(Some(RData::A(rdata::A(RESOLVED_ADDR))));

        log::debug!(
            "Spoofing query for captive portal domain: {}",
            return_query.name()
        );

        let lookup = Lookup::new_with_deadline(
            return_query,
            Arc::new([return_record]),
            Instant::now() + Duration::from_secs(3),
        );
        Ok(Box::new(ForwardLookup(lookup)) as Box<_>)
    }

    /// Determines whether a DNS query is allowable. Currently, this implies that the query is
    /// either a `A` or a `CNAME` query for `captive.apple.com`.
    fn is_captive_portal_domain(query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && ALLOWED_DOMAINS.contains(query.name())
    }

    /// Forward DNS queries to the specified DNS resolver.
    async fn resolve_forward(
        resolver: TokioAsyncResolver,
        query: LowerQuery,
    ) -> std::result::Result<Box<dyn LookupObject>, ResolveError> {
        log::info!("!!!");
        log::info!("!!! resolve_forward: {query:?}");
        log::info!("!!!");

        let return_query = query.original().clone();

        let lookup = match tokio::time::timeout(
            Duration::from_secs(5),
            resolver.lookup(return_query.name().clone(), return_query.query_type()),
        )
        .await
        {
            Ok(asdf) => asdf,
            Err(_err) => {
                return Ok(Box::new(EmptyLookup) as Box<dyn LookupObject>);
            }
        };

        lookup.map(|lookup| Box::new(ForwardLookup(lookup)) as Box<_>)
    }
}

/// A handle to control a DNS resolver.
///
/// When all resolver handles are dropped, the resolver will stop.
#[derive(Clone)]
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
    pub async fn enable_forward(&self, dns_servers: Vec<IpAddr>) {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = self.tx.unbounded_send(ResolverMessage::SetConfig {
            new_config: Config::Forwarding { dns_servers },
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
}

impl LocalResolver {
    /// Constructs a new filtering resolver and it's handle.
    async fn new() -> Result<(Self, ResolverHandle), Error> {
        let (command_tx, command_rx) = mpsc::unbounded();
        let command_tx = Arc::new(command_tx);
        let weak_tx = Arc::downgrade(&command_tx);

        let (socket, cleanup_ifconfig) = Self::new_random_socket().await?;
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

            log::error!("DNS server exited!");
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
    ) -> Result<ServerFuture<ResolverImpl>, Error> {
        let mut server = ServerFuture::new(ResolverImpl { tx: command_tx });

        server.register_socket(socket);

        Ok(server)
    }

    /// Create a new [net::UdpSocket] bound to port 53 on loopback.
    ///
    /// This socket will try to bind to the following IPs in sequential order:
    /// - random ip in the range 127.1-255.0-255.0-255 : 53
    /// - random ip in the range 127.1-255.0-255.0-255 : 53
    /// - random ip in the range 127.1-255.0-255.0-255 : 53
    /// - 127.0.0.1 : 53
    ///
    /// We do this to try and avoid collisions with other DNS servers running on the same system.
    ///
    /// # Returns
    /// - The first successfully bound [UdpSocket]
    /// - An [OnDrop] guard that will delete the IP aliases added, if any.
    ///   If the guard is dropped while the socket is in use, calls to read/write will likely fail.
    async fn new_random_socket() -> Result<(UdpSocket, OnDrop), Error> {
        use std::net::Ipv4Addr;

        let random_loopback = || async move {
            //let addr = Ipv4Addr::new(127, 1u8.max(random()), random(), random());
            // FIXME: test
            let addr = Ipv4Addr::new(127, 31, 31, 31);

            // TODO: this command requires root privileges and will thus not work in `cargo test`.
            // This means that the tests will fall back to 127.0.0.1, and will not assert that the
            // ifconfig stuff actually works. We probably do want to test this, so what do?
            let output = Command::new("ifconfig")
                .args([LOOPBACK, "alias", &format!("{addr}"), "up"])
                .output()
                .await
                .inspect_err(|e| {
                    log::warn!("Failed to spawn `ifconfig {LOOPBACK} alias {addr} up`: {e}")
                })
                .ok()?;

            if !output.status.success() {
                log::warn!("Non-zero exit code from ifconfig: {}", output.status);
                return None;
            }

            log::debug!("Created loopback address {addr}");

            // Clean up ip address when stopping the resolver
            let cleanup_ifconfig = on_drop(move || {
                tokio::task::spawn(async move {
                    log::debug!("Cleaning up loopback address {addr}");

                    let result = Command::new("ifconfig")
                        .args([LOOPBACK, "delete", &format!("{addr}")])
                        .output()
                        .await;

                    if let Err(e) = result {
                        log::warn!("Failed to clean up {LOOPBACK} alias {addr}: {e}");
                    }
                });
            })
            .boxed();

            Some((addr, cleanup_ifconfig))
        };

        for attempt in 0.. {
            let (socket_addr, on_drop) = match attempt {
                ..3 => match random_loopback().await {
                    Some(random) => random,
                    None => continue,
                },
                3 => (Ipv4Addr::LOCALHOST, OnDrop::noop()),
                4.. => break,
            };

            match net::UdpSocket::bind((socket_addr, DNS_PORT)).await {
                Ok(socket) => return Ok((socket, on_drop)),
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

        while let Some(request) = self.rx.next().await {
            match request {
                ResolverMessage::SetConfig {
                    new_config,
                    response_tx,
                } => {
                    log::debug!("Updating config: {new_config:?}");

                    self.update_config(new_config);
                    flush_system_cache();
                    let _ = response_tx.send(());
                }
                ResolverMessage::Query {
                    dns_query,
                    response_tx,
                } => {
                    self.inner_resolver.resolve(dns_query, response_tx);
                }
            }
        }
    }

    /// Update the current DNS config.
    fn update_config(&mut self, config: Config) {
        match config {
            Config::Blocking => self.blocking(),
            Config::Forwarding { mut dns_servers } => {
                // make sure not to accidentally forward queries to ourselves
                dns_servers.retain(|addr| *addr != self.bound_to.ip());
                self.forwarding(dns_servers);
            }
        }
    }

    /// Turn into a blocking resolver.
    fn blocking(&mut self) {
        self.inner_resolver = Resolver::Blocking;
    }

    /// Turn into a forwarding resolver (forward DNS queries to [dns_servers]).
    fn forwarding(&mut self, dns_servers: Vec<IpAddr>) {
        let forward_server_config =
            NameServerConfigGroup::from_ips_clear(&dns_servers, DNS_PORT, true);

        let forward_config = ResolverConfig::from_parts(None, vec![], forward_server_config);
        let resolver_opts = ResolverOpts::default();

        let resolver = TokioAsyncResolver::tokio(forward_config, resolver_opts);

        self.inner_resolver = Resolver::Forwarding(resolver);
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

type LookupResponse<'a> = MessageResponse<
    'a,
    'a,
    Box<dyn Iterator<Item = &'a Record> + Send + 'a>,
    std::iter::Empty<&'a Record>,
    std::iter::Empty<&'a Record>,
    std::iter::Empty<&'a Record>,
>;

/// An implementation of [hickory_server::server::RequestHandler] that forwards queries to
/// `FilteringResolver`.
struct ResolverImpl {
    tx: Weak<mpsc::UnboundedSender<ResolverMessage>>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a dyn LookupObject,
    ) -> LookupResponse<'a> {
        let mut response_header = Header::new();
        response_header.set_id(message.id());
        response_header.set_op_code(OpCode::Query);
        response_header.set_message_type(MessageType::Response);
        response_header.set_authoritative(false);

        MessageResponseBuilder::from_message_request(message).build(
            response_header,
            lookup.iter(),
            // forwarder responses only contain query answers, no ns,soa or additionals
            std::iter::empty(),
            std::iter::empty(),
            std::iter::empty(),
        )
    }

    /// This function is called when a DNS query is sent to the local resolver
    async fn lookup<R: ResponseHandler>(&self, message: &Request, mut response_handler: R) {
        if let Some(tx_ref) = self.tx.upgrade() {
            let mut tx = (*tx_ref).clone();
            let query = message.query();
            let (response_tx, response_rx) = oneshot::channel();
            let _ = tx
                .send(ResolverMessage::Query {
                    dns_query: query.clone(),
                    response_tx,
                })
                .await;

            let lookup_result = response_rx.await;
            let response_result = match lookup_result {
                Ok(Ok(ref lookup)) => {
                    let response = Self::build_response(message, lookup.as_ref());
                    response_handler.send_response(response).await
                }
                Err(_error) => return,
                Ok(Err(resolve_err)) => match resolve_err.kind() {
                    ResolveErrorKind::NoRecordsFound { response_code, .. } => {
                        let response = MessageResponseBuilder::from_message_request(message)
                            .error_msg(message.header(), *response_code);
                        response_handler.send_response(response).await
                    }
                    _other => {
                        let response = Self::build_response(message, &EmptyLookup);
                        response_handler.send_response(response).await
                    }
                },
            };
            if let Err(err) = response_result {
                log::error!("Failed to send response: {err}");
            }
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for ResolverImpl {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        if !request.src().ip().is_loopback() {
            log::error!("Dropping a stray request from outside: {}", request.src());
            return Header::new().into();
        }
        if let MessageType::Query = request.message_type() {
            match request.op_code() {
                OpCode::Query => {
                    self.lookup(request, response_handle).await;
                }
                _ => {
                    log::trace!("Dropping non-query request: {:?}", request);
                }
            };
        }

        return Header::new().into();
    }
}

struct ForwardLookup(Lookup);

/// This trait has to be reimplemented for the Lookup so that it can be sent back to the
/// RequestHandler implementation.
impl LookupObject for ForwardLookup {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Record> + Send + 'a> {
        Box::new(self.0.record_iter())
    }

    fn take_additionals(&mut self) -> Option<Box<dyn LookupObject>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hickory_server::resolver::{
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        TokioAsyncResolver,
    };
    use std::{mem, net::UdpSocket, sync::Mutex, thread, time::Duration};

    /// Can't have multiple local resolvers running at the same time, as they will try to bind to
    /// the same address and port. The tests below use this lock to run sequentially.
    static LOCK: Mutex<()> = Mutex::new(());

    async fn start_resolver() -> ResolverHandle {
        super::start_resolver().await.unwrap()
    }

    fn get_test_resolver(addr: SocketAddr) -> hickory_server::resolver::TokioAsyncResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[addr.ip()], addr.port(), true),
        );
        TokioAsyncResolver::tokio(resolver_config, ResolverOpts::default())
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
            Ok::<(), hickory_server::resolver::error::ResolveError>(())
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

    #[test_log::test]
    fn test_shutdown() {
        let _mutex = LOCK.lock().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.block_on(start_resolver());
        let addr = handle.listening_addr();
        mem::drop(handle);
        thread::sleep(Duration::from_millis(300));
        UdpSocket::bind(addr).expect("Failed to bind to a port that should have been removed");
    }
}
