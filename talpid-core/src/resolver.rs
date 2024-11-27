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
use std::sync::LazyLock;

/// If a local DNS resolver should be used at all times.
///
/// This setting does not affect the error or blocked state. In those states, we will want to use
/// the local DNS resoler to work around Apple's captive portals check. Exactly how this is done is
/// documented elsewhere.
pub static LOCAL_DNS_RESOLVER: LazyLock<bool> = LazyLock::new(|| {
    use talpid_platform_metadata::MacosVersion;
    let version = MacosVersion::new().expect("Could not detect macOS version");
    let v = |s| MacosVersion::from_raw_version(s).unwrap();
    // Apple services tried to perform DNS lookups on the physical interface on some macOS
    // versions, so we added redirect rules to always redirect DNS to our local DNS resolver.
    // This seems to break some apps which do not like that we redirect DNS on port 53 to our local
    // DNS resolver running on some other, arbitrary port, and so we disable this behaviour on
    // macOS versions that are unaffected by this naughty bug.
    //
    // The workaround should only be applied to the affected macOS versions because some programs
    // set the `skip filtering` pf flag on loopback, which meant that the pf filtering would break
    // unexpectedly. We could clear the `skip filtering` flag to force pf filtering on loopback,
    // but apparently it is good practice to enable `skip filtering` on loopback so we decided
    // against this. Source: https://www.openbsd.org/faq/pf/filter.html
    //
    // It should be noted that most programs still works fine with this workaround enabled. Notably
    // programs that use `getaddrinfo` would behave correctly when we redirect DNS to our local
    // resolver, while some programs always used port 53 no matter what (nslookup for example).
    // Also, most programs don't set the `skip filtering` pf flag on loopback, but some notable
    // ones do for some reason. Orbstack is one such example, which meant that people running
    // containers would run into the aforementioned issue.
    let use_local_dns_resolver = v("14.6") <= version && version < v("15.1");
    if use_local_dns_resolver {
        log::debug!("Using local DNS resolver");
    }
    use_local_dns_resolver
});

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
    UdpBindError(#[source] io::Error),

    /// Failed to get local address of a bound UDP socket
    #[error("Failed to get local address of a bound UDP socket")]
    GetSocketAddrError(#[source] io::Error),
}

/// A DNS resolver that forwards queries to some other DNS server
///
/// Is controlled by commands sent through [ResolverHandle]s.
struct LocalResolver {
    rx: mpsc::UnboundedReceiver<ResolverMessage>,
    dns_server: Option<(tokio::task::JoinHandle<()>, oneshot::Receiver<()>)>,
    inner_resolver: Resolver,
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

impl From<Config> for Resolver {
    fn from(mut config: Config) -> Self {
        match &mut config {
            Config::Blocking => Resolver::Blocking,
            Config::Forwarding { dns_servers } => {
                // make sure not to accidentally forward queries to ourselves
                dns_servers.retain(|addr| !addr.is_loopback());

                let forward_server_config =
                    NameServerConfigGroup::from_ips_clear(dns_servers, 53, true);

                let forward_config =
                    ResolverConfig::from_parts(None, vec![], forward_server_config);
                let resolver_opts = ResolverOpts::default();

                let resolver = TokioAsyncResolver::tokio(forward_config, resolver_opts);

                Resolver::Forwarding(resolver)
            }
        }
    }
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
        let return_query = query.original().clone();

        let lookup = resolver
            .lookup(return_query.name().clone(), return_query.query_type())
            .await;

        lookup.map(|lookup| Box::new(ForwardLookup(lookup)) as Box<_>)
    }
}

/// A handle to control a DNS resolver.
///
/// When all resolver handles are dropped, the resolver will stop.
#[derive(Clone)]
pub struct ResolverHandle {
    tx: Arc<mpsc::UnboundedSender<ResolverMessage>>,
    listening_port: u16,
}

impl ResolverHandle {
    fn new(tx: Arc<mpsc::UnboundedSender<ResolverMessage>>, listening_port: u16) -> Self {
        Self { tx, listening_port }
    }

    /// Get listening port for resolver handle
    pub fn listening_port(&self) -> u16 {
        self.listening_port
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
        let (tx, rx) = mpsc::unbounded();
        let command_tx = Arc::new(tx);

        let weak_tx = Arc::downgrade(&command_tx);
        let (mut server, port) = Self::new_server(0, weak_tx.clone()).await?;

        let (server_done_tx, server_done_rx) = oneshot::channel();
        let server_handle = tokio::spawn(async move {
            loop {
                if let Err(err) = server.block_until_done().await {
                    log::error!("DNS server unexpectedly stopped: {}", err);

                    if weak_tx.strong_count() > 0 {
                        log::debug!("Attempting restart server");
                        match Self::new_server(port, weak_tx.clone()).await {
                            Ok((new_server, _port)) => {
                                server = new_server;
                                continue;
                            }
                            Err(error) => {
                                log::error!("Failed to restart DNS server: {error}");
                            }
                        }
                    }
                }
                break;
            }

            let _ = server_done_tx.send(());
        });

        let resolver = Self {
            rx,
            dns_server: Some((server_handle, server_done_rx)),
            inner_resolver: Resolver::from(Config::Blocking),
        };

        Ok((resolver, ResolverHandle::new(command_tx, port)))
    }

    async fn new_server(
        port: u16,
        command_tx: Weak<mpsc::UnboundedSender<ResolverMessage>>,
    ) -> Result<(ServerFuture<ResolverImpl>, u16), Error> {
        let mut server = ServerFuture::new(ResolverImpl { tx: command_tx });

        let server_listening_socket =
            tokio::net::UdpSocket::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port))
                .await
                .map_err(Error::UdpBindError)?;
        let port = server_listening_socket
            .local_addr()
            .map_err(Error::GetSocketAddrError)?
            .port();
        server.register_socket(server_listening_socket);

        Ok((server, port))
    }

    /// Runs the filtering resolver as an actor, listening for new queries instances.  When all
    /// related [ResolverHandle] instances are dropped, this function will return, closing the DNS
    /// server.
    async fn run(mut self) {
        while let Some(request) = self.rx.next().await {
            match request {
                ResolverMessage::SetConfig {
                    new_config,
                    response_tx,
                } => {
                    log::debug!("Updating config: {new_config:?}");

                    self.inner_resolver = Resolver::from(new_config);
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

        if let Some((server_handle, done_rx)) = self.dns_server.take() {
            server_handle.abort();
            let _ = done_rx.await;
        }
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
    use std::{mem, net::UdpSocket, thread, time::Duration};

    async fn start_resolver() -> ResolverHandle {
        super::start_resolver().await.unwrap()
    }

    fn get_test_resolver(port: u16) -> hickory_server::resolver::TokioAsyncResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[Ipv4Addr::LOCALHOST.into()], port, true),
        );
        TokioAsyncResolver::tokio(resolver_config, ResolverOpts::default())
    }

    #[test]
    fn test_successful_lookup() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.block_on(start_resolver());
        let test_resolver = get_test_resolver(handle.listening_port());

        rt.block_on(async move {
            for domain in &*ALLOWED_DOMAINS {
                test_resolver.lookup(domain, RecordType::A).await?;
            }
            Ok::<(), hickory_server::resolver::error::ResolveError>(())
        })
        .expect("Resolution of domains failed");
    }

    #[test]
    fn test_failed_lookup() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.block_on(start_resolver());
        let test_resolver = get_test_resolver(handle.listening_port());

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

    #[test]
    fn test_shutdown() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.block_on(start_resolver());
        let port = handle.listening_port();
        mem::drop(handle);
        thread::sleep(Duration::from_millis(300));
        UdpSocket::bind((Ipv4Addr::LOCALHOST, port))
            .expect("Failed to bind to a port that should have been removed");
    }
}
