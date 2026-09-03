#[cfg(target_os = "android")]
pub use crate::https_client::SocketBypassRequest;
use crate::{
    DnsResolver, access::AccessTokenStore, availability::ApiAvailability,
    https_client::HttpsConnector, proxy::ConnectionModeProvider,
};
use duplicate::duplicate_item;
use futures::{
    TryFutureExt as _,
    channel::{mpsc, oneshot},
    stream::StreamExt,
};
use http_body_util::{BodyExt, Full};
use hyper::{
    Method, Uri,
    body::{Body, Buf, Bytes, Incoming},
    client::conn::http1,
    header::{self, HeaderValue},
};
use mullvad_types::account::AccountNumber;
use std::{
    convert::Infallible,
    error::Error as StdError,
    future::pending,
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::Notify,
    time::{sleep, timeout},
};
use tracing::{Level, instrument};

pub use hyper::StatusCode;

const USER_AGENT: &str = "mullvad-app";

pub type Result<T> = std::result::Result<T, Error>;

/// Default timeout for a [`Request`].
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Deadline to establish a TLS connection to the remote.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Describes all the ways a REST request can fail
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("REST client service is down")]
    RestServiceDown,

    #[error("Request cancelled")]
    Aborted,

    #[error("Hyper error")]
    HyperError(#[from] Arc<hyper::Error>),

    #[error("Invalid header value")]
    InvalidHeaderError,

    #[error("Connection failed")]
    Connect(#[source] Arc<io::Error>),

    #[error("DNS lookup failed")]
    Dns(#[source] Arc<io::Error>),

    #[error("HTTP error")]
    HttpError(#[from] Arc<http::Error>),

    #[error("Request timed out")]
    TimeoutError,

    #[error("Failed to deserialize data")]
    DeserializeError(#[from] Arc<serde_json::Error>),

    /// Unexpected response code
    #[error("Unexpected response status code {0} - {1}")]
    ApiError(StatusCode, String),

    /// The string given was not a valid URI.
    #[error("Not a valid URI {0}")]
    InvalidUri(#[from] Arc<http::uri::InvalidUri>),

    #[error("Set account number on factory with no access token store")]
    NoAccessTokenStore,

    /// Failed to obtain versions
    #[error("Failed to obtain versions")]
    FetchVersions(#[from] Arc<anyhow::Error>),

    /// Body exceeded size limit
    #[error("Body exceeded size limit")]
    BodyTooLarge,
}

impl From<Infallible> for Error {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            Error::HyperError(_) | Error::TimeoutError | Error::Connect(..) | Error::Dns(..)
        )
    }

    /// Return true if there was no route to the destination
    pub fn is_offline(&self) -> bool {
        match self {
            Error::Connect(error) | Error::Dns(error)
                if let Some(cause) = error.source()
                    && let Some(err) = cause.downcast_ref::<std::io::Error>() =>
            {
                err.raw_os_error() == Some(libc::ENETUNREACH)
            }
            _ => false,
        }
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self, Error::Aborted)
    }

    /// Returns a new instance for which `abortable_stream::Aborted` is mapped to `Self::Aborted`.
    fn map_aborted(self) -> Self {
        // Hyper returs `cancelled` if the request or underlying connection was dropped before it
        // was started. `is_user` is returned when the underlying connection is dropped while
        // the request is in-flight, but it may also be true for other errors triggered by us.
        if let Error::HyperError(error) = &self
            && (error.is_canceled() || error.is_user())
        {
            return Self::Aborted;
        }
        self
    }
}

/// An actor service for sending HTTP requests.
///
/// The service is tied to a specific host, specified in [RequestService::spawn].
/// It allows for on-demand termination of in-flight requests.
///
/// TLS connections are established by [`HttpsConnector`] and may be reused for multiple request.
pub(crate) struct RequestService<C> {
    host: Arc<str>,
    requests_rx: mpsc::UnboundedReceiver<SendRequest>,
    access_tokens: AccessTokenStore,
    /// [`Notify`] that is used to signal whether [`Connection`] should be closed.
    reset: Arc<Notify>,
    /// The active HTTP connection to the server, if any.
    connection: Option<Connection>,
    dns_resolver: Arc<dyn DnsResolver>,
    connector: HttpsConnector,
    connection_mode_provider: C,
    api_availability: ApiAvailability,
}

struct Connection {
    send_request: http1::SendRequest<Full<Bytes>>,
    idle_timeout: Duration,
    connection_task: tokio::task::AbortHandle,
}

/// A handle to interact with a spawned `RequestService`.
#[derive(Debug, Clone)]
pub struct RequestServiceHandle {
    tx: Arc<mpsc::UnboundedSender<SendRequest>>,
    reset: Arc<Notify>,
    host: Arc<str>,
}

#[derive(Debug)]
pub struct SendRequest {
    request: Request<Full<Bytes>>,
    response_tx: oneshot::Sender<std::result::Result<Response<Incoming>, Error>>,
}

impl<C: ConnectionModeProvider + 'static> RequestService<C> {
    /// Constructs a new [`RequestService`].
    pub fn spawn(
        host: impl Into<Arc<str>>,
        api_availability: ApiAvailability,
        connection_mode_provider: C,
        dns_resolver: Arc<dyn DnsResolver>,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
        #[cfg(any(feature = "api-override", test))] disable_tls: bool,
    ) -> RequestServiceHandle {
        let connection_mode = connection_mode_provider.initial();

        let connector = HttpsConnector::new(
            connection_mode,
            #[cfg(target_os = "android")]
            socket_bypass_tx.clone(),
            #[cfg(any(feature = "api-override", test))]
            disable_tls,
        );

        let (command_tx, command_rx) = mpsc::unbounded();
        let command_tx = Arc::new(command_tx);
        let reset = Arc::new(Notify::new());
        let host = host.into();
        let service = Self {
            host: Arc::clone(&host),
            dns_resolver,
            requests_rx: command_rx,
            reset: Arc::clone(&reset),
            connector,
            connection_mode_provider,
            api_availability,
            connection: None,
            access_tokens: Default::default(),
        };
        let handle = RequestServiceHandle {
            host,
            tx: command_tx,
            reset,
        };
        tokio::spawn(service.run());
        handle
    }

    async fn run(mut self) {
        let reset = self.reset.clone();

        loop {
            let evict_connection = async {
                match &self.connection {
                    None => pending().await,
                    Some(c) => {
                        // TODO: this does not take into account in-flight `Body`s
                        sleep(c.idle_timeout).await;
                        c.idle_timeout
                    }
                }
            };

            tokio::select! {
                // Handle reset-commands
                _ = reset.notified() => self.close_connection(),

                // Handle change API access method
                new_mode = self.connection_mode_provider.receive() => {
                    let Some(new_mode) = new_mode else { break };
                    self.soft_close_connection();
                    self.connector.set_connection_mode(new_mode);
                }

                // Handle request
                request = self.requests_rx.next() => {
                    let Some(SendRequest { request, response_tx }) = request else { break };

                    let result = tokio::select! {
                        r = self.send_request(request) => r,
                        // Handle reset-commands during request processing.
                        _ = reset.notified() => {
                            self.close_connection();
                            Err(Error::Aborted)
                        }
                    };
                    let _ = response_tx.send(result);
                }

                // Evict idle connections
                idle_for = evict_connection => {
                    tracing::trace!("Connection idle for {:.02}s. Evicting.", idle_for.as_secs_f32());
                    self.soft_close_connection();
                }
            }
        }
    }

    /// Resolve `self.host` an IP and port. This uses the internal [`DnsResolver`].
    #[instrument(level = Level::TRACE, skip(self), ret)]
    async fn resolve(&self) -> Result<SocketAddr> {
        const DEFAULT_PORT: u16 = 443;

        tracing::trace!("resolving {:?}", self.host);
        self.dns_resolver
            .resolve(self.host.to_string())
            .await
            .map(|addrs| addrs.first().copied())
            .and_then(|addr| addr.ok_or_else(|| io::Error::other("Empty DNS response")))
            .map(|mut addr| {
                if addr.port() == 0 {
                    addr.set_port(DEFAULT_PORT);
                }
                addr
            })
            .map_err(Arc::new)
            .map_err(Error::Dns)
    }

    async fn connect(&mut self) -> Result<Connection> {
        let addr = self.resolve().await?;
        tracing::trace!("connecting to {addr} ({:?})", self.host);
        let idle_timeout = self.connector.idle_timeout();
        let connection = self
            .connector
            .connect(addr, &self.host)
            .await
            .map_err(Arc::new)
            .map_err(Error::Connect)?;
        let (send_request, connection) = http1::handshake(connection).await?;

        Ok(Connection {
            send_request,
            idle_timeout,
            connection_task: tokio::spawn(connection).abort_handle(),
        })
    }

    async fn send_request(&mut self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>> {
        let uri = request.uri().clone();

        let t = request.timeout;

        let future = self
            .send_request_inner(request)
            .map_err(|error| error.map_aborted());

        let result = timeout(t, future)
            .await
            .map_err(|_timeout| Error::TimeoutError)
            .flatten();

        // TODO: retry once if error is due to access token expiry

        if let Err(err) = &result
            && err.is_network_error()
            && !self.api_availability.is_offline()
        {
            tracing::warn!("{uri:?} request failed: {err:?}");
            self.close_connection();
            self.connection_mode_provider.rotate().await;
        }

        result
    }

    async fn send_request_inner(
        &mut self,
        mut request: Request<Full<Bytes>>,
    ) -> Result<Response<Incoming>> {
        let _ = self.api_availability.wait_for_unsuspend().await;

        let connection = match &mut self.connection {
            Some(sr) => sr,
            None => {
                let connection = timeout(CONNECT_TIMEOUT, self.connect())
                    .map_err(|_elapsed| Error::TimeoutError)
                    .await??;
                self.connection.insert(connection)
            }
        };

        if let Some(account) = &request.account {
            let access_token = self
                .access_tokens
                .get_token(account, &self.host, &mut connection.send_request)
                .await?;
            let auth = HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|_| Error::InvalidHeaderError)?;
            request
                .request
                .headers_mut()
                .insert(header::AUTHORIZATION, auth);
        }

        let response = connection
            .send_request
            .send_request(request.request)
            .await
            .map_err(Error::from)?;

        if !request.expected_status.contains(&response.status()) {
            if !request.expected_status.is_empty() {
                tracing::error!(
                    "Unexpected HTTP status code {}, expected codes [{}]",
                    response.status(),
                    request
                        .expected_status
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
            if !response.status().is_success() {
                let Err(e) = handle_error_response(response).await;

                // TODO: SMELL: Hitting this code-path has a dependency on the user specifying expected_status correctly
                if let Some(account) = &request.account {
                    self.access_tokens.check_err(account, &e);
                }

                return Err(e);
            }
        }

        Ok(Response::new(response))
    }
}

impl<C> RequestService<C> {
    /// Drop [`RequestService::connection`] and abort [`Connection::connection_task`].
    ///
    /// Any in-flight [`Body`] returned [`Self::send_request`] will be aborted.
    fn close_connection(&mut self) {
        if let Some(connection) = self.connection.take() {
            connection.connection_task.abort();
        }
    }

    /// Drop [`RequestService::connection`], preventing it to be used for new requests.
    ///
    /// This does not directly abort [`Connection::connection_task`], which keeps in-flight
    /// [`Body`]s active.
    fn soft_close_connection(&mut self) {
        self.connection = None;
    }
}

impl<C> Drop for RequestService<C> {
    fn drop(&mut self) {
        self.close_connection();
    }
}

impl RequestServiceHandle {
    /// Resets the corresponding RequestService, dropping all in-flight requests.
    pub fn reset(&self) {
        self.reset.notify_one();
    }

    /// Submits a [`Request`] for execution to the request service.
    async fn dispatch(&self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(SendRequest {
                request,
                response_tx,
            })
            .map_err(|_| Error::RestServiceDown)?;
        response_rx.await.map_err(|_| Error::RestServiceDown)?
    }

    /// Get a [`RequestFactory`] that can be used to construct HTTP requests for dispatch on this [`RequestService`].
    pub fn request(&self) -> RequestFactory {
        RequestFactory {
            service: self.clone(),
            default_timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// A REST request that is sent to the [`RequestService`] to be executed.
#[derive(Debug)]
pub struct Request<B> {
    request: hyper::Request<B>,
    timeout: Duration,
    service: RequestServiceHandle,
    account: Option<AccountNumber>,
    expected_status: &'static [hyper::StatusCode],
}

impl<B: Body> Request<B> {
    fn new(request: hyper::Request<B>, service: RequestServiceHandle) -> Self {
        Self {
            request,
            timeout: DEFAULT_TIMEOUT,
            service,
            account: None,
            expected_status: &[],
        }
    }

    /// Set the account number to obtain authentication for.
    ///
    /// If set, the [`RequestService`] may preempt this request with a request to fetch an access token.
    pub fn account(mut self, account: AccountNumber) -> Result<Self> {
        self.account = Some(account);
        Ok(self)
    }

    /// Sets timeout for the request.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn expected_status(mut self, expected_status: &'static [hyper::StatusCode]) -> Self {
        self.expected_status = expected_status;
        self
    }

    pub fn header<T: header::IntoHeaderName>(mut self, key: T, value: &str) -> Result<Self> {
        let header_value =
            http::HeaderValue::from_str(value).map_err(|_| Error::InvalidHeaderError)?;
        self.request.headers_mut().insert(key, header_value);
        Ok(self)
    }

    /// Returns the URI of the request
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }
}

/// Dispatch the [`Request`] to the [`RequestService`].
impl IntoFuture for Request<Full<Bytes>> {
    type Output = Result<Response<Incoming>>;

    // TODO: When ATPIT is stablized, switch to `impl Future`
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.service.clone().dispatch(self).await })
    }
}

/// Successful result of a REST request
#[derive(Debug)]
pub struct Response<B> {
    response: hyper::Response<B>,
}

impl<B: Body + Unpin> Response<B>
where
    Error: From<<B as Body>::Error>,
{
    fn new(response: hyper::Response<B>) -> Self {
        Self { response }
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    pub fn headers(&self) -> &hyper::HeaderMap<HeaderValue> {
        self.response.headers()
    }

    pub async fn deserialize<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        deserialize_body_inner(self.response).await
    }

    pub async fn body(self) -> Result<Vec<u8>> {
        Ok(BodyExt::collect(self.response).await?.to_bytes().to_vec())
    }

    pub async fn body_with_max_size(self, size_limit: usize) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = vec![];
        let mut stream = self.response.into_data_stream();

        while let Some(chunk) = stream.next().await {
            data.extend(chunk?.chunk());
            if data.len() > size_limit {
                return Err(Error::BodyTooLarge);
            }
        }

        Ok(data)
    }
}

#[derive(serde::Deserialize)]
struct OldErrorResponse {
    pub code: String,
}

/// If `NewErrorResponse::type` is not defined it should default to "about:blank"
const DEFAULT_ERROR_TYPE: &str = "about:blank";
#[derive(Debug, serde::Deserialize)]
struct NewErrorResponse {
    pub r#type: Option<String>,
}

#[derive(Clone)]
pub struct RequestFactory {
    service: RequestServiceHandle,
    default_timeout: Duration,
}

impl RequestFactory {
    pub fn request<B: Body + Default>(&self, path: &str, method: Method) -> Result<Request<B>> {
        Ok(Request::new(
            self.hyper_request(path, method, B::default())?,
            self.service.clone(),
        )
        .timeout(self.default_timeout))
    }

    #[duplicate_item(
        method    METHOD;
        [get]     [GET];
        [post]    [POST];
        [put]     [PUT];
        [delete]  [DELETE];
        [head]    [HEAD];
    )]
    pub fn method(&self, path: &str) -> Result<Request<Full<Bytes>>> {
        self.request(path, Method::METHOD)
    }

    pub fn post_json<S: serde::Serialize>(
        &self,
        path: &str,
        body: &S,
    ) -> Result<Request<Full<Bytes>>> {
        self.json_request(Method::POST, path, body)
    }

    pub fn post_json_bytes(&self, path: &str, body: Vec<u8>) -> Result<Request<Full<Bytes>>> {
        self.json_request_with_bytes(Method::POST, path, body)
    }

    pub fn put_json<S: serde::Serialize>(
        &self,
        path: &str,
        body: &S,
    ) -> Result<Request<Full<Bytes>>> {
        self.json_request(Method::PUT, path, body)
    }

    pub fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    fn json_request_with_bytes(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
    ) -> Result<Request<Full<Bytes>>> {
        let request = hyper_request_json_bytes(&self.service.host, path, method, body)?;
        Ok(Request::new(request, self.service.clone()).timeout(self.default_timeout))
    }

    fn json_request<S: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &S,
    ) -> Result<Request<Full<Bytes>>> {
        let body = serde_json::to_vec(&body)?;
        self.json_request_with_bytes(method, path, body)
    }

    fn hyper_request<B>(&self, path: &str, method: Method, body: B) -> Result<http::Request<B>> {
        hyper_request(&self.service.host, path, method, body)
    }
}

pub(crate) fn hyper_request<B>(
    host: &str,
    path: &str,
    method: Method,
    body: B,
) -> Result<http::Request<B>> {
    let uri = format!("https://{host}/{path}");
    let request = http::request::Builder::new()
        .method(method)
        .uri(uri)
        .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
        .header(header::ACCEPT, HeaderValue::from_static("application/json"))
        .header(
            header::HOST,
            HeaderValue::from_str(host).map_err(|_| Error::InvalidHeaderError)?,
        )
        .body(body)?;
    Ok(request)
}

pub(crate) fn hyper_request_json_bytes(
    host: &str,
    path: &str,
    method: Method,
    body: Vec<u8>,
) -> Result<http::Request<Full<Bytes>>> {
    let body = Full::new(Bytes::from(body));
    let mut request = hyper_request(host, path, method, body)?;

    let headers = request.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    Ok(request)
}

fn get_body_length<B>(response: &hyper::Response<B>) -> usize {
    response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|length| length.parse::<usize>().ok())
        .unwrap_or(0)
}

// TODO: replace Infallible with ! when Rust 1.100 is stabilized
pub(crate) async fn handle_error_response<B: Body>(
    response: hyper::Response<B>,
) -> Result<Infallible>
where
    Error: From<B::Error>,
{
    let status = response.status();
    let error_message = match status {
        hyper::StatusCode::METHOD_NOT_ALLOWED => "Method not allowed",
        status => match get_body_length(&response) {
            0 => status.canonical_reason().unwrap_or("Unexpected error"),
            _length => {
                return match response.headers().get("content-type") {
                    Some(content_type) if content_type == "application/problem+json" => {
                        // TODO: We should make sure we unify the new error format and the old
                        // error format so that they both produce the same Errors for the same
                        // problems after being processed.
                        let err: NewErrorResponse = deserialize_body_inner(response).await?;
                        // The new error type replaces the `code` field with the `type` field.
                        // This is what is used to programmatically check the error.
                        Err(Error::ApiError(
                            status,
                            err.r#type
                                .unwrap_or_else(|| String::from(DEFAULT_ERROR_TYPE)),
                        ))
                    }
                    _ => {
                        let err: OldErrorResponse = deserialize_body_inner(response).await?;
                        Err(Error::ApiError(status, err.code))
                    }
                };
            }
        },
    };
    Err(Error::ApiError(status, error_message.to_owned()))
}

async fn deserialize_body_inner<T, B>(response: hyper::Response<B>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
    B: Body,
    Error: From<B::Error>,
{
    use http_body_util::BodyExt;

    let collected = BodyExt::collect(response).await?;
    let res = serde_json::from_slice(&collected.to_bytes())?;
    Ok(res)
}

#[derive(Clone)]
pub struct MullvadRestHandle {
    pub(crate) service: RequestServiceHandle,
    pub(crate) factory: RequestFactory,
    pub availability: ApiAvailability,
}

impl MullvadRestHandle {
    pub(crate) fn new(
        service: RequestServiceHandle,
        factory: RequestFactory,
        availability: ApiAvailability,
    ) -> Self {
        Self {
            service,
            factory,
            availability,
        }
    }

    pub fn service(&self) -> RequestServiceHandle {
        self.service.clone()
    }
}

impl Deref for MullvadRestHandle {
    type Target = RequestFactory;
    fn deref(&self) -> &Self::Target {
        &self.factory
    }
}

impl DerefMut for MullvadRestHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.factory
    }
}

macro_rules! impl_into_arc_err {
    ($ty:ty) => {
        impl From<$ty> for Error {
            fn from(error: $ty) -> Self {
                Error::from(Arc::from(error))
            }
        }
    };
}

impl_into_arc_err!(hyper::Error);
impl_into_arc_err!(serde_json::Error);
impl_into_arc_err!(http::Error);
impl_into_arc_err!(http::uri::InvalidUri);
