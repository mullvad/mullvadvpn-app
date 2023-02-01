use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, Weak},
};

use std::time::{Duration, Instant};

use futures::{
    channel::{mpsc, oneshot},
    SinkExt, StreamExt,
};

use trust_dns_server::{
    authority::{
        EmptyLookup, LookupObject, MessageRequest, MessageResponse, MessageResponseBuilder,
    },
    client::{
        op::LowerQuery,
        rr::{LowerName, RecordType},
    },
    proto::{
        op::{header::MessageType, op_code::OpCode, Header},
        rr::{domain::Name, record_data::RData, Record},
    },
    resolver::lookup::Lookup,
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    ServerFuture,
};

const ALLOWED_RECORD_TYPES: &[RecordType] = &[RecordType::A, RecordType::AAAA, RecordType::CNAME];
const CAPTIVE_PORTAL_DOMAINS: &[&str] = &["captive.apple.com", "netcts.cdn-apple.com"];

lazy_static::lazy_static! {
    static ref ALLOWED_DOMAINS: Vec<LowerName> =
        CAPTIVE_PORTAL_DOMAINS
            .iter()
            .map(|domain| LowerName::from(Name::from_str(domain).unwrap()))
            .collect();
}

const TTL_SECONDS: u32 = 3;
/// An IP address to be used in the DNS response to the captive domain query. The address itself
/// belongs to the documentation range so should never be reachable.
const RESOLVED_ADDR: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 1);

/// Starts a resolver. Returns a cloneable handle, which can activate, deactivate and shut down the
/// resolver. When all instances of a handle are dropped, the server will stop.
pub(crate) async fn start_resolver() -> Result<ResolverHandle, Error> {
    let (resolver, resolver_handle) = FilteringResolver::new().await?;
    tokio::spawn(resolver.run());
    Ok(resolver_handle)
}

/// Resolver errors
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to bind UDP socket
    #[error(display = "Failed to bind UDP socket")]
    UdpBindError(#[error(source)] io::Error),

    /// Failed to get local address of a bound UDP socket
    #[error(display = "Failed to get local address of a bound UDP socket")]
    GetSocketAddrError(#[error(source)] io::Error),
}

/// A filtering resolver. Listens on a specified port for DNS queries and responds queries for
/// `catpive.apple.com`. Can be toggled to unbind, be bound but not respond or bound and responding
/// to some queries.
struct FilteringResolver {
    rx: mpsc::Receiver<ResolverMessage>,
    dns_server: Option<(tokio::task::JoinHandle<()>, oneshot::Receiver<()>)>,
}

/// The `FilteringResolver` is an actor responding to DNS queries.
type ResolverMessage = (LowerQuery, oneshot::Sender<Box<dyn LookupObject>>);

/// A handle to control a filtering resolver. When all resolver handles are dropped, custom
/// resolver will stop.
#[derive(Clone)]
pub(crate) struct ResolverHandle {
    _tx: Arc<mpsc::Sender<ResolverMessage>>,
    listening_port: u16,
}

impl ResolverHandle {
    fn new(tx: Arc<mpsc::Sender<ResolverMessage>>, listening_port: u16) -> Self {
        Self {
            _tx: tx,
            listening_port,
        }
    }

    /// Get listening port for resolver handle
    pub fn listening_port(&self) -> u16 {
        self.listening_port
    }
}

impl FilteringResolver {
    /// Constructs a new filtering resolver and it's handle.
    async fn new() -> Result<(Self, ResolverHandle), Error> {
        let (tx, rx) = mpsc::channel(0);
        let command_tx = Arc::new(tx);

        let mut server = ServerFuture::new(ResolverImpl {
            tx: Arc::downgrade(&command_tx),
        });

        let server_listening_socket =
            tokio::net::UdpSocket::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0))
                .await
                .map_err(Error::UdpBindError)?;
        let port = server_listening_socket
            .local_addr()
            .map_err(Error::GetSocketAddrError)?
            .port();
        server.register_socket(server_listening_socket);

        let (server_done_tx, server_done_rx) = oneshot::channel();
        let server_handle = tokio::spawn(async move {
            if let Err(err) = server.block_until_done().await {
                log::error!("DNS server stopped: {}", err);
            }

            let _ = server_done_tx.send(());
        });
        let resolver = Self {
            rx,
            dns_server: Some((server_handle, server_done_rx)),
        };

        Ok((resolver, ResolverHandle::new(command_tx, port)))
    }

    /// Runs the filtering resolver as an actor, listening for new queries instances.  When all
    /// related [ResolverHandle] instances are dropped, this function will return, closing the DNS
    /// server.
    async fn run(mut self) {
        while let Some((query, tx)) = self.rx.next().await {
            self.resolve(query, tx);
        }

        if let Some((server_handle, done_rx)) = self.dns_server.take() {
            server_handle.abort();
            let _ = done_rx.await;
        }
    }

    /// Resolvers a query to nothing or a documentation address
    fn resolve(&mut self, query: LowerQuery, tx: oneshot::Sender<Box<dyn LookupObject>>) {
        if !self.allow_query(&query) {
            let _ = tx.send(Box::new(EmptyLookup) as Box<dyn LookupObject>);
            return;
        }

        let return_query = query.original().clone();
        let mut return_record = Record::with(
            return_query.name().clone(),
            return_query.query_type(),
            TTL_SECONDS,
        );
        return_record.set_data(Some(RData::A(RESOLVED_ADDR)));

        let lookup = Lookup::new_with_deadline(
            return_query,
            Arc::new([return_record]),
            Instant::now() + Duration::from_secs(3),
        );
        let _ = tx.send(Box::new(ForwardLookup(lookup)));
    }

    /// Determines whether a DNS query is allowable. Currently, this implies that the query is
    /// either a `A`, `AAAA` or a `CNAME` query for `captive.apple.com`.
    fn allow_query(&self, query: &LowerQuery) -> bool {
        ALLOWED_RECORD_TYPES.contains(&query.query_type()) && ALLOWED_DOMAINS.contains(query.name())
    }
}

/// An implementation of [trust_dns_server::server::RequestHandler] that forwards queries to
/// `FilteringResolver`.
struct ResolverImpl {
    tx: Weak<mpsc::Sender<ResolverMessage>>,
}

impl ResolverImpl {
    fn build_response<'a>(
        message: &'a MessageRequest,
        lookup: &'a mut Box<dyn LookupObject>,
    ) -> MessageResponse<
        'a,
        'a,
        Box<dyn Iterator<Item = &'a Record> + Send + 'a>,
        std::iter::Empty<&'a Record>,
        std::iter::Empty<&'a Record>,
        std::iter::Empty<&'a Record>,
    > {
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

    async fn lookup<R: ResponseHandler>(&self, message: &Request, mut response_handler: R) {
        if let Some(tx_ref) = self.tx.upgrade() {
            let mut tx = (&*tx_ref).clone();
            let query = message.query();
            let (lookup_tx, lookup_rx) = oneshot::channel();
            let _ = tx.send((query.clone(), lookup_tx)).await;
            let mut lookup_result: Box<dyn LookupObject> = lookup_rx
                .await
                .unwrap_or_else(|_| Box::new(EmptyLookup) as Box<dyn LookupObject>);
            let response = Self::build_response(&message, &mut lookup_result);

            if let Err(err) = response_handler.send_response(response).await {
                log::error!("Failed to send response: {}", err);
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
    use std::{mem, net::UdpSocket, thread, time::Duration};
    use trust_dns_server::resolver::{
        config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
        TokioAsyncResolver,
    };

    async fn start_resolver() -> ResolverHandle {
        super::start_resolver().await.unwrap()
    }

    fn get_test_resolver(port: u16) -> trust_dns_server::resolver::TokioAsyncResolver {
        let resolver_config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(&[Ipv4Addr::LOCALHOST.into()], port, true),
        );
        TokioAsyncResolver::tokio(resolver_config, ResolverOpts::default()).unwrap()
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
            Ok::<(), trust_dns_server::resolver::error::ResolveError>(())
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
