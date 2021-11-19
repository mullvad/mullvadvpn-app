use socket2::{Domain, Socket, Type};

use std::{
    collections::BTreeSet,
    ffi::CString,
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    str::FromStr,
    sync::{Arc, Mutex, Weak},
};

#[cfg(target_os = "macos")]
use std::{
    net,
    num::NonZeroU32,
    os::unix::io::{FromRawFd, IntoRawFd, RawFd},
};

use futures::{
    channel::{mpsc, oneshot},
    future::Either,
    SinkExt, StreamExt,
};

use crate::tunnel_state_machine::TunnelCommand;
use trust_dns_server::{
    authority::{
        EmptyLookup, LookupObject, MessageRequest, MessageResponse, MessageResponseBuilder,
    },
    client::{
        op::LowerQuery,
        rr::{LowerName, RecordType},
    },
    proto::{
        self,
        iocompat::AsyncIoTokioAsStd,
        op::{header::MessageType, op_code::OpCode, Header},
        rr::{domain::Name, record_data::RData, Record},
        TokioTime,
    },
    resolver::{
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        error::ResolveError,
        lookup::Lookup,
        name_server::{GenericConnection, GenericConnectionProvider},
        AsyncResolver,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    ServerFuture,
};

const ALLOWED_RECORD_TYPES: &[RecordType] = &[RecordType::A, RecordType::AAAA, RecordType::CNAME];
const CAPTIVE_PORTAL_DOMAIN: &str = "captive.apple.com";

type TunnelCommandSender = Weak<mpsc::UnboundedSender<TunnelCommand>>;

pub(crate) async fn start_resolver(
    sender: TunnelCommandSender,
    exclusion_gid: Option<u32>,
) -> Result<ResolverHandle, Error> {
    start_resolver_inner(sender, exclusion_gid, 53).await
}

async fn start_resolver_inner(
    sender: TunnelCommandSender,
    exclusion_gid: Option<u32>,
    port: u16,
) -> Result<ResolverHandle, Error> {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || run_resolver(sender, tx, exclusion_gid, port));
    rx.await.map_err(|_| Error::LauncherThreadPanic)?
}

fn run_resolver(
    tunnel_tx: TunnelCommandSender,
    done_tx: oneshot::Sender<Result<ResolverHandle, Error>>,
    exclusion_gid: Option<u32>,
    port: u16,
) {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.worker_threads(2);
    builder.max_blocking_threads(1);
    builder.on_thread_start(move || {
        #[cfg(target_os = "macos")]
        if let Some(gid) = exclusion_gid.clone() {
            let ret = unsafe { libc::setgid(gid) };
            if ret != 0 {
                log::error!("Failed to set group ID");
                return;
            }
        } else {
            return;
        }
    });
    let rt = builder.build().expect("failed to initialize tokio runtime");
    match rt.block_on(FilteringResolver::new(tunnel_tx, port)) {
        Ok((resolver, resolver_handle)) => {
            let _ = done_tx.send(Ok(resolver_handle));
            rt.block_on(resolver.run());
        }
        Err(err) => {
            let _ = done_tx.send(Err(err));
        }
    }
}
/// Resolver errors
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to launch resolver
    #[error(display = "Failed to launch resolver")]
    LaunchResolver(#[error(source)] ResolveError),

    /// Failed to bind TCP socket
    #[error(display = "Failed to bind TCP socket")]
    TcpBindError(#[error(source)] io::Error),

    /// Failed to bind UDP socket
    #[error(display = "Failed to bind UDP socket")]
    UdpBindError(#[error(source)] io::Error),

    /// Launcher thread panicked
    #[error(display = "Panic in the launcher thread")]
    LauncherThreadPanic,

    /// The resolver has already shut down
    #[error(display = "Resolver is already shut down")]
    ResolverShutdown,

    /// Failed to obtain system resolvers
    #[error(display = "Failed to obtain system resolvers")]
    NoSystemResolvers,
}

struct FilteringResolver {
    excluded_resolver: ExcludedUpstreamResolver,
    rx: mpsc::Receiver<ResolverMessage>,
    resolver_state: ResolverState,
    tunnel_tx: TunnelCommandSender,
    dns_server: Option<(tokio::task::JoinHandle<()>, oneshot::Receiver<()>)>,
    command_sender: Weak<mpsc::Sender<ResolverMessage>>,
    runtime_provider: RuntimeProvider,
    port: u16,
}

type OurConnectionProvider = GenericConnectionProvider<RuntimeProvider>;
type ExcludedUpstreamResolver = AsyncResolver<GenericConnection, OurConnectionProvider>;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ResolverState {
    Active(Option<(String, Vec<IpAddr>)>),
    Inactive,
    Shutdown,
}

impl ResolverState {
    fn is_running(&self) -> bool {
        match self {
            Self::Active(_) => true,
            _ => false,
        }
    }
}

pub(crate) enum ResolverMessage {
    Request(LowerQuery, oneshot::Sender<Box<dyn LookupObject>>),
    SetResolverState(
        ResolverState,
        oneshot::Sender<Result<ResolverStateToggleResult, Error>>,
    ),
}

pub(crate) struct ResolverStateToggleResult {
    pub currently_used_resolvers: BTreeSet<IpAddr>,
    unblock_tx: oneshot::Sender<()>,
}

impl ResolverStateToggleResult {
    fn new(resolvers: &[IpAddr]) -> (Self, oneshot::Receiver<()>) {
        let (unblock_tx, rx) = oneshot::channel();
        (
            Self {
                currently_used_resolvers: resolvers.iter().cloned().collect(),
                unblock_tx,
            },
            rx,
        )
    }

    pub fn unblock(self) {
        let _ = self.unblock_tx.send(());
    }
}

#[derive(Clone)]
pub(crate) struct ResolverHandle {
    tx: Arc<mpsc::Sender<ResolverMessage>>,
}

impl ResolverHandle {
    fn new(tx: Arc<mpsc::Sender<ResolverMessage>>) -> Self {
        Self { tx }
    }

    /// Enable the resolver
    pub async fn set_active(
        &self,
        config: Option<(String, Vec<IpAddr>)>,
    ) -> Result<ResolverStateToggleResult, Error> {
        self.set_state(ResolverState::Active(config)).await
    }

    pub async fn set_inactive(&self) -> Result<ResolverStateToggleResult, Error> {
        self.set_state(ResolverState::Inactive).await
    }

    pub async fn shutdown(&self) -> Result<ResolverStateToggleResult, Error> {
        self.set_state(ResolverState::Shutdown).await
    }

    async fn set_state(&self, state: ResolverState) -> Result<ResolverStateToggleResult, Error> {
        let (done_tx, done_rx) = oneshot::channel();
        let tx: &mpsc::Sender<ResolverMessage> = &*self.tx;
        let mut tx = tx.clone();
        tx.send(ResolverMessage::SetResolverState(state, done_tx))
            .await
            .map_err(|_| Error::ResolverShutdown)?;

        done_rx.await.map_err(|_| Error::ResolverShutdown)?
    }
}

impl FilteringResolver {
    async fn new(
        tunnel_tx: TunnelCommandSender,
        port: u16,
    ) -> Result<(Self, ResolverHandle), Error> {
        let (tx, rx) = mpsc::channel(0);
        let command_tx = Arc::new(tx);

        let runtime_provider = RuntimeProvider::new();

        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[], 53, false),
        );
        let resolver = ExcludedUpstreamResolver::new(
            resolver_config.clone(),
            ResolverOpts::default(),
            runtime_provider.clone(),
        )
        .map_err(Error::LaunchResolver)?;

        let resolver = Self {
            excluded_resolver: resolver,
            resolver_state: ResolverState::Shutdown,
            rx,
            tunnel_tx,
            command_sender: Arc::downgrade(&command_tx),
            dns_server: None,
            runtime_provider,
            port,
        };

        Ok((resolver, ResolverHandle::new(command_tx)))
    }

    async fn run(mut self) {
        use ResolverMessage::*;
        while let Some(message) = self.rx.next().await {
            match message {
                Request(query, tx) => {
                    if self.resolver_state.is_running() {
                        tokio::spawn(self.resolve(query, tx));
                    }
                }
                SetResolverState(resolver_state, tx) => {
                    match resolver_state {
                        ResolverState::Shutdown => {
                            self.stop_server().await;
                        }
                        running_state => {
                            if self.dns_server.is_none() {
                                if let Err(err) = self.spawn_new_server().await {
                                    let _ = tx.send(Err(err));
                                    let _ = self.reset_resolver().await;
                                    continue;
                                }
                            }
                            self.resolver_state = running_state;
                        }
                    }
                    match self.reset_resolver().await {
                        Ok(new_resolvers) => {
                            let (result, unblock_rx) =
                                ResolverStateToggleResult::new(&new_resolvers);
                            let _ = tx.send(Ok(result));
                            let _ = unblock_rx.await;
                        }
                        Err(err) => {
                            let _ = tx.send(Err(err));
                        }
                    }
                }
            }
        }

        std::mem::drop(self);
    }

    async fn spawn_new_server(&mut self) -> Result<(), Error> {
        self.stop_server().await;
        if let Some(tx) = self.command_sender.upgrade() {
            let resolver_handle = ResolverImpl { tx };
            let mut server = ServerFuture::new(resolver_handle);
            let listening_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.port);
            let udp_sock = tokio::net::UdpSocket::bind(listening_addr)
                .await
                .map_err(Error::UdpBindError)?;
            let tcp_sock = tokio::net::TcpListener::bind(listening_addr)
                .await
                .map_err(Error::TcpBindError)?;
            server.register_socket(udp_sock);
            server.register_listener(tcp_sock, std::time::Duration::from_secs(1));

            let (server_done_tx, server_done_rx) = oneshot::channel();
            let server_handle = tokio::spawn(async move {
                if let Err(err) = server.block_until_done().await {
                    log::error!("DNS server stopped: {}", err);
                }
                let _ = server_done_tx.send(());
            });

            self.dns_server = Some((server_handle, server_done_rx));
        }
        Ok(())
    }

    async fn stop_server(&mut self) {
        if let Some((old_server, done_rx)) = self.dns_server.take() {
            old_server.abort();
            if done_rx.await.is_err() {
                log::error!("Server future was already stopped");
            }
        }
    }

    async fn reset_resolver(&mut self) -> Result<Vec<IpAddr>, Error> {
        log::trace!("Resetting custom resolver");
        let (best_interface, resolver_addresses) = self.get_resolver_config();
        self.runtime_provider.update_best_interface(best_interface);
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(resolver_addresses, 53, false),
        );
        let mut resolver_options = ResolverOpts::default();
        resolver_options.preserve_intermediates = true;
        let resolver = AsyncResolver::new(
            resolver_config.clone(),
            resolver_options,
            self.runtime_provider.clone(),
        )
        .map_err(Error::LaunchResolver)?;
        let resolver_addresses = resolver_addresses.to_vec();
        self.excluded_resolver = resolver;
        Ok(resolver_addresses)
    }

    fn get_resolver_config(&self) -> (&str, &[IpAddr]) {
        match &self.resolver_state {
            ResolverState::Active(ref resolvers) => {
                // TODO: actually pick the best resolver
                resolvers
                    .as_ref()
                    .filter(|(_, addresses)| {
                        !addresses.iter().any(|ip| ip.is_loopback())
                    })
                    .map(|(interface_name, addresses)| (interface_name.as_str(), addresses.as_slice()))
                    .unwrap_or(("", &[]))
            }
            _ => ("", &[]),
        }
    }

    fn resolve(
        &mut self,
        query: LowerQuery,
        tx: oneshot::Sender<Box<dyn LookupObject>>,
    ) -> impl Future<Output = ()> {
        let empty_response = Box::new(EmptyLookup) as Box<dyn LookupObject>;
        if !self.should_service_request(&query) {
            let _ = tx.send(empty_response);
            return Either::Left(async {});
        }

        log::trace!("Looking up {}", query.name());

        let unblock_tx = self.tunnel_tx.clone();
        let lookup: Box<dyn Future<Output = Result<Lookup, ResolveError>> + Unpin + Send> =
            Box::new(self.excluded_resolver.lookup(
                query.name().clone(),
                query.query_type(),
                Default::default(),
            ));
        let resolver_state = self.resolver_state.clone();
        Either::Right(async move {
            match lookup.await {
                Ok(result) => {
                    let lookup = ForwardLookup(result);
                    let ip_records = lookup
                        .iter()
                        .filter_map(|record| match record.rdata() {
                            RData::A(ipv4) => Some(IpAddr::from(*ipv4)),
                            RData::AAAA(ipv6) => Some(IpAddr::from(*ipv6)),
                            _ => None,
                        })
                        .collect::<BTreeSet<_>>();

                    if !ip_records.is_empty() {
                        if resolver_state.is_running() {
                            Self::unblock_ips(unblock_tx, ip_records).await;
                        }
                    }
                    if tx.send(Box::new(lookup)).is_err() {
                        log::error!("Failed to send response to resolver");
                    }
                }
                Err(err) => {
                    log::trace!("Failed to resolve {}: {}", query, err);
                    let _ = tx.send(empty_response);
                }
            }
        })
    }

    async fn unblock_ips(maybe_tx: TunnelCommandSender, addresses: BTreeSet<IpAddr>) {
        let (done_tx, done_rx) = oneshot::channel();
        if maybe_tx
            .upgrade()
            .and_then(|tx| {
                tx.unbounded_send(TunnelCommand::AddAllowedIps(addresses, done_tx))
                    .ok()
            })
            .is_some()
        {
            let _ = done_rx.await;
        } else {
            log::error!("Failed to send IPs to unblocker");
        }
    }

    fn should_service_request(&self, query: &LowerQuery) -> bool {
        self.resolver_state.is_running() && self.allow_query(query)
    }

    fn allow_query(&self, query: &LowerQuery) -> bool {
        let captive_apple_com: LowerName =
            LowerName::from(Name::from_str(CAPTIVE_PORTAL_DOMAIN).unwrap());
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && query.name() == &captive_apple_com
    }
}

struct ResolverImpl {
    tx: Arc<mpsc::Sender<ResolverMessage>>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a mut Box<dyn LookupObject>,
    ) -> MessageResponse<'a, 'a> {
        let mut response_header = Header::new();
        response_header.set_id(message.id());
        response_header.set_op_code(OpCode::Query);
        response_header.set_message_type(MessageType::Response);
        response_header.set_authoritative(false);

        MessageResponseBuilder::from_message_request(message).build(
            response_header,
            lookup.iter(),
            // forwarder responses only contain query answers, no ns,soa or additionals
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _> + Send>,
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _> + Send>,
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _> + Send>,
        )
    }

    async fn lookup<R: ResponseHandler>(&self, message: &Request, mut response_handler: R) {
        let tx_ref: &mpsc::Sender<ResolverMessage> = &*self.tx;
        let mut tx = tx_ref.clone();

        let query = message.query();
        let (lookup_tx, lookup_rx) = oneshot::channel();
        let _ = tx
            .send(ResolverMessage::Request(query.clone(), lookup_tx))
            .await;
        let mut lookup_result: Box<dyn LookupObject> = lookup_rx
            .await
            .unwrap_or_else(|_| Box::new(EmptyLookup) as Box<dyn LookupObject>);
        let response = Self::build_response(&message, &mut lookup_result);

        if let Err(err) = response_handler.send_response(response).await {
            log::error!("Failed to send response: {}", err);
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

#[derive(Clone)]
struct RuntimeProvider {
    best_interface: Arc<Mutex<Option<NonZeroU32>>>,
}

impl RuntimeProvider {
    fn new() -> Self {
        Self {
            best_interface: Arc::new(Mutex::new(None)),
        }
    }

    fn update_best_interface(&self, best_interface: &str) {
        let ifname = match CString::new(best_interface) {
            Ok(name) => name,
            Err(err) => {
                log::error!("Failed to construct an interface name CString: {}", err);
                return;
            }
        };
        if let Some(index) = NonZeroU32::new(unsafe { libc::if_nametoindex(ifname.as_ptr()) }) {
            *self.best_interface.lock().unwrap() = Some(index);
        }
    }
}

impl proto::runtime_provider::RuntimeProvider for RuntimeProvider {
    type UdpSocket = tokio::net::UdpSocket;
    type TcpConnection = AsyncIoTokioAsStd<tokio::net::TcpStream>;
    type Time = TokioTime;

    fn connect_tcp(
        &self,
        addr: SocketAddr,
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::TcpConnection>> + Send>> {
        let best_interface = self.best_interface.clone();

        Box::pin(async move {
            let raw_fd = open_socket(addr, Type::STREAM, socket2::Protocol::TCP, best_interface)?;

            let socket = unsafe { tokio::net::TcpSocket::from_raw_fd(raw_fd) };
            socket.connect(addr).await.map(AsyncIoTokioAsStd)
        })
    }

    fn bind_udp(
        &self,
        addr: SocketAddr,
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::UdpSocket>> + Send>> {
        let best_interface = self.best_interface.clone();
        Box::pin(async move {
            let raw_fd = open_socket(
                addr,
                socket2::Type::DGRAM,
                socket2::Protocol::UDP,
                best_interface.clone(),
            )?;

            let std_socket = unsafe { net::UdpSocket::from_raw_fd(raw_fd) };
            tokio::net::UdpSocket::from_std(std_socket)
        })
    }

    fn spawn_bg<F>(&self, f: F)
    where
        F: Future<Output = Result<(), trust_dns_server::proto::error::ProtoError>> + Send + 'static,
    {
        tokio::spawn(f);
    }
}

fn open_socket(
    addr: SocketAddr,
    sock_type: Type,
    protocol: socket2::Protocol,
    best_interface: Arc<Mutex<Option<NonZeroU32>>>,
) -> io::Result<RawFd> {
    let socket = Socket::new(Domain::for_address(addr), sock_type, Some(protocol))?;

    socket.set_nonblocking(true)?;

    match best_interface
        .lock()
        .expect("best interface lock poisoned")
        .as_ref()
    {
        Some(iface_index) => {
            if let Err(err) = socket.bind_device_by_index(Some(*iface_index)) {
                log::error!("Failed to bind by index: {}", err);
                return Err(err);
            }
        }
        None => {
            log::error!("Failed to get best interface index");
        }
    };
    Ok(socket.into_raw_fd())
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
    use std::{fs, net::UdpSocket, process::Command};
    use subslice::SubsliceExt;

    fn random_port() -> u16 {
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.local_addr().unwrap().port()
    }

    const NAMESERVER: &[u8] = b"nameserver";

    fn read_resolvconf() -> Option<(String, Vec<IpAddr>)> {
        let contents = fs::read("/etc/resolv.conf").unwrap();
        let nameserver_index = contents
            .find(NAMESERVER)
            .expect("Failed to read /etc/resolv.conf");
        let end = contents[nameserver_index..]
            .find(b"\n")
            .expect("no \n after nameserver")
            + nameserver_index;
        let ip_addr_subslice = &contents[nameserver_index + NAMESERVER.len()..end];

        let resolver_ip =
            IpAddr::from_str(std::str::from_utf8(ip_addr_subslice).unwrap().trim()).unwrap();
        let route_output = String::from_utf8(
            Command::new("route")
                .arg("get")
                .arg(resolver_ip.to_string())
                .output()
                .expect("Failed to run 'route get'")
                .stdout,
        )
        .unwrap();

        let mut output_parts = route_output.split_whitespace();
        while let Some(part) = output_parts.next() {
            if part.trim() == "interface:" {
                return Some((output_parts.next().unwrap().to_string(), vec![resolver_ip]));
            }
        }
        panic!("Couldn't deduce interface")
    }

    async fn start_resolver() -> (
        ResolverHandle,
        u16,
        mpsc::UnboundedReceiver<TunnelCommand>,
        Arc<mpsc::UnboundedSender<TunnelCommand>>,
    ) {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let tx = Arc::new(tx);
        let port = random_port();

        let resolver_handle = super::start_resolver_inner(Arc::downgrade(&tx), None, port)
            .await
            .unwrap();
        (resolver_handle, port, rx, tx)
    }

    async fn get_test_resolver(port: u16) -> trust_dns_server::resolver::TokioAsyncResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[Ipv4Addr::LOCALHOST.into()], port, true),
        );
        AsyncResolver::new(
            resolver_config,
            ResolverOpts::default(),
            proto::TokioRuntime,
        )
        .unwrap()
    }

    #[test]
    fn test_successful_lookup() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (handle, port, mut cmd_rx, _txx) = rt.block_on(start_resolver());
        let test_resolver = rt.block_on(get_test_resolver(port));
        let resolver_config = read_resolvconf();
        rt.block_on(async {
            let unblocker = handle
                .set_active(resolver_config)
                .await
                .expect("failed to make resovler active");
            unblocker.unblock();
        });

        let captive_portal_domain = LowerName::from(Name::from_str(CAPTIVE_PORTAL_DOMAIN).unwrap());
        let resolver_result = rt.block_on(async move {
            let dns_request =
                test_resolver.lookup(captive_portal_domain, RecordType::A, Default::default());
            let unblock_request = cmd_rx.next();

            use futures::future::Either;
            match futures::future::select(dns_request, unblock_request).await {
                Either::Left((_resolution_result, _unblock_request_future)) => {
                    panic!("DNS response recieved before unblocking request")
                }
                Either::Right((unblock_request, resolution)) => {
                    std::mem::drop(unblock_request);
                    resolution.await
                }
            }
        });
        resolver_result.expect("Failed to resolve test domain");
    }

    #[test]
    fn test_failed_lookup_when_active() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let (handle, port, mut cmd_rx, _tx) = rt.block_on(start_resolver());
        let test_resolver = rt.block_on(get_test_resolver(port));

        let resolver_config = read_resolvconf();
        rt.block_on(async {
            let unblocker = handle
                .set_active(resolver_config)
                .await
                .expect("failed to make resovler active");
            unblocker.unblock();
        });

        let captive_portal_domain = LowerName::from(Name::from_str("apple.com").unwrap());
        let resolver_result = rt.block_on(async move {
            let dns_request =
                test_resolver.lookup(captive_portal_domain, RecordType::A, Default::default());
            let unblock_request = cmd_rx.next();

            use futures::future::Either;
            match futures::future::select(dns_request, unblock_request).await {
                Either::Left((dns_response, _unblock_request_future)) => dns_response,
                Either::Right((_unblock_request, _resolution)) => {
                    panic!(
                        "There should be no unblocking for a request that shouldn't be serviced"
                    );
                }
            }
        });
        assert!(
            resolver_result.is_err(),
            "Non-whitelisted DNS request should fail"
        )
    }

    #[test]
    fn test_failed_lookup_when_inactive() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let (handle, port, mut cmd_rx, _tx) = rt.block_on(start_resolver());
        let test_resolver = rt.block_on(get_test_resolver(port));

        rt.block_on(async {
            let unblocker = handle
                .set_inactive()
                .await
                .expect("failed to make resovler active");
            unblocker.unblock();
        });

        let captive_portal_domain = LowerName::from(Name::from_str("apple.com").unwrap());
        let resolver_result = rt.block_on(async move {
            let dns_request =
                test_resolver.lookup(captive_portal_domain, RecordType::A, Default::default());
            let unblock_request = cmd_rx.next();

            use futures::future::Either;
            match futures::future::select(dns_request, unblock_request).await {
                Either::Left((dns_response, _unblock_request_future)) => {
                    dns_response
                }
                Either::Right((_unblock_request, _resolution)) => {
                    panic!("There should be no unblocking for for a request when the resolver is inactive");
                }
            }

        });
        assert!(
            resolver_result.is_err(),
            "Non-whitelisted DNS request should fail"
        )
    }

    #[test]
    fn test_unbinding() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let (handle, port, mut _cmd_rx, _tx) = rt.block_on(start_resolver());
        let server_sockaddr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);

        let _ = UdpSocket::bind(server_sockaddr)
            .expect("Failed to bind to resolver socket addr when it should be unbound");

        rt.block_on(async {
            let unblocker = handle
                .set_inactive()
                .await
                .expect("failed to make resovler active");
            unblocker.unblock();
        });

        assert!(UdpSocket::bind(server_sockaddr).is_err());

        rt.block_on(async {
            let unblocker = handle
                .shutdown()
                .await
                .expect("failed to make resovler active");
            unblocker.unblock();
        });

        UdpSocket::bind(server_sockaddr)
            .expect("Failed to bind to resolver socket addr when it should be unbound");
    }
}
