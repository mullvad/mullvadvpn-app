use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
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
use libc::{c_void, kill, pid_t, proc_listallpids, proc_pidpath, SIGHUP};
use std::sync::LazyLock;

const ALLOWED_RECORD_TYPES: &[RecordType] =
    &[RecordType::A, /*RecordType::AAAA,*/ RecordType::CNAME];
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
pub(crate) async fn start_resolver() -> Result<ResolverHandle, Error> {
    let (resolver, resolver_handle) = ForwardingResolver::new().await?;
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

/// A forwarding resolver
struct ForwardingResolver {
    rx: mpsc::UnboundedReceiver<ResolverMessage>,
    dns_server: Option<(tokio::task::JoinHandle<()>, oneshot::Receiver<()>)>,
    forward_resolver: LocalResolver,
}

/// Resolver message
enum ResolverMessage {
    /// Set config
    SetConfig {
        /// New DNS config to use
        new_config: LocalConfig,
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },
    /// Query
    Query(
        LowerQuery,
        oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
    ),
}

/// Resolver config
#[derive(Debug, Default, Clone)]
enum LocalConfig {
    /// Drop DNS queries. For captive portal domains, return faux records
    #[default]
    Blocked,
    /// Forward DNS queries to a configured server
    ForwardDns {
        /// Remote DNS server to use
        dns_servers: Vec<IpAddr>,
    },
}

enum LocalResolver {
    /// Drop DNS queries. For captive portal domains, return faux records
    Blocked,
    /// Forward DNS queries to a configured server
    ForwardDns(TokioAsyncResolver),
}

impl From<LocalConfig> for LocalResolver {
    fn from(config: LocalConfig) -> Self {
        match config {
            LocalConfig::Blocked => LocalResolver::Blocked,
            LocalConfig::ForwardDns { ref dns_servers } => {
                let forward_server_config =
                    NameServerConfigGroup::from_ips_clear(dns_servers, 53, true);

                let forward_config =
                    ResolverConfig::from_parts(None, vec![], forward_server_config);
                let resolver_opts = ResolverOpts::default();

                let resolver = TokioAsyncResolver::tokio(forward_config, resolver_opts);

                LocalResolver::ForwardDns(resolver)
            }
        }
    }
}

impl LocalResolver {
    pub fn resolve(
        &self,
        query: LowerQuery,
        tx: oneshot::Sender<std::result::Result<Box<dyn LookupObject>, ResolveError>>,
    ) {
        let lookup = match self {
            LocalResolver::Blocked => Either::Left(Self::resolve_blocked(query)),
            LocalResolver::ForwardDns(resolver) => {
                Either::Right(Self::resolve_forward(resolver.clone(), query))
            }
        };

        tokio::spawn(async move {
            let _ = tx.send(lookup.await);
        });
    }

    /// Resolution in blocked state will return spoofed records for captive portal domains.
    async fn resolve_blocked(
        query: LowerQuery,
    ) -> std::result::Result<Box<dyn LookupObject>, ResolveError> {
        if !Self::is_captive_portal_domain(&query) {
            log::trace!("Ignoring query: {query}");
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
        log::trace!(
            "Resolving query: {}, {}",
            return_query.name(),
            return_query.query_type()
        );

        let lookup = resolver
            .lookup(return_query.name().clone(), return_query.query_type())
            .await;

        lookup.map(|lookup| Box::new(ForwardLookup(lookup)) as Box<_>)
    }
}

/// A handle to control a forwarding resolver. When all resolver handles are dropped, custom
/// resolver will stop.
#[derive(Clone)]
pub(crate) struct ResolverHandle {
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

    /// Set the DNS server to forward queries to
    pub async fn enable_forward(&self, dns_servers: Vec<IpAddr>) {
        let dns_servers = dns_servers
            .into_iter()
            .filter(|addr| !addr.is_loopback())
            .collect();

        let (response_tx, response_rx) = oneshot::channel();
        let _ = self.tx.unbounded_send(ResolverMessage::SetConfig {
            new_config: LocalConfig::ForwardDns { dns_servers },
            response_tx,
        });

        let _ = response_rx.await;
    }

    // Disable forwarding
    pub async fn disable_forward(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        let _ = self.tx.unbounded_send(ResolverMessage::SetConfig {
            new_config: LocalConfig::Blocked,
            response_tx,
        });

        let _ = response_rx.await;
    }
}

impl ForwardingResolver {
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
            forward_resolver: LocalResolver::from(LocalConfig::Blocked),
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

                    self.forward_resolver = LocalResolver::from(new_config);
                    flush_system_cache();
                    let _ = response_tx.send(());
                }
                ResolverMessage::Query(query, tx) => {
                    self.forward_resolver.resolve(query, tx);
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
    if let Some(mdns_pid) = pid_of_path(MDNS_RESPONDER_PATH) {
        if unsafe { kill(mdns_pid as i32, SIGHUP) } != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

/// Return the first process identifier matching a specified path, if one exists.
fn pid_of_path(find_path: impl AsRef<Path>) -> Option<pid_t> {
    match list_pids() {
        Ok(pids) => {
            for pid in pids {
                if let Ok(path) = process_path(pid) {
                    if path == find_path.as_ref() {
                        return Some(pid);
                    }
                }
            }
            None
        }
        Err(error) => {
            log::error!("Failed to list processes: {error}");
            None
        }
    }
}

/// Obtain a list of all pids
fn list_pids() -> io::Result<Vec<pid_t>> {
    // SAFETY: Passing in null and 0 returns the number of processes
    let num_pids = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if num_pids <= 0 {
        return Err(io::Error::last_os_error());
    }
    let num_pids = usize::try_from(num_pids).unwrap();
    let mut pids = vec![0i32; num_pids];

    let buf_sz = (num_pids * std::mem::size_of::<pid_t>()) as i32;
    // SAFETY: 'pids' is large enough to contain 'num_pids' processes
    let num_pids = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut c_void, buf_sz) };
    if num_pids == -1 {
        return Err(io::Error::last_os_error());
    }

    pids.resize(usize::try_from(num_pids).unwrap(), 0);

    Ok(pids)
}

fn process_path(pid: pid_t) -> io::Result<PathBuf> {
    let mut buffer = [0u8; libc::MAXPATHLEN as usize];
    // SAFETY: `proc_pidpath` returns at most `buffer.len()` bytes
    let buf_len = unsafe {
        proc_pidpath(
            pid,
            buffer.as_mut_ptr() as *mut c_void,
            u32::try_from(buffer.len()).unwrap(),
        )
    };
    if buf_len == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(PathBuf::from(
        std::str::from_utf8(&buffer[0..buf_len as usize])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid process path"))?,
    ))
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
            let (lookup_tx, lookup_rx) = oneshot::channel();
            let _ = tx
                .send(ResolverMessage::Query(query.clone(), lookup_tx))
                .await;

            let lookup_result = lookup_rx.await;
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
