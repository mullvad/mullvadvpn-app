#[cfg(target_os = "android")]
pub use crate::https_client_with_sni::SocketBypassRequest;
use crate::{
    access::AccessTokenStore,
    address_cache::AddressCache,
    availability::ApiAvailabilityHandle,
    https_client_with_sni::{HttpsConnectorWithSni, HttpsConnectorWithSniHandle},
    proxy::ApiConnectionMode,
};
use futures::{
    channel::{mpsc, oneshot},
    stream::StreamExt,
    Stream,
};
use hyper::{
    client::{connect::Connect, Client},
    header::{self, HeaderValue},
    Method, Uri,
};
use mullvad_types::account::AccountToken;
use std::{
    error::Error as StdError,
    str::FromStr,
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_types::ErrorExt;

#[cfg(feature = "api-override")]
use crate::API;

pub use hyper::StatusCode;

const USER_AGENT: &str = "mullvad-app";

const API_IP_CHECK_INITIAL: Duration = Duration::from_secs(15 * 60);
const API_IP_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const API_IP_CHECK_ERROR_INTERVAL: Duration = Duration::from_secs(15 * 60);

pub type Result<T> = std::result::Result<T, Error>;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Describes all the ways a REST request can fail
#[derive(err_derive::Error, Debug, Clone)]
pub enum Error {
    #[error(display = "REST client service is down")]
    RestServiceDown,

    #[error(display = "Request cancelled")]
    Aborted,

    #[error(display = "Hyper error")]
    HyperError(#[error(source)] Arc<hyper::Error>),

    #[error(display = "Invalid header value")]
    InvalidHeaderError,

    #[error(display = "HTTP error")]
    HttpError(#[error(source)] Arc<http::Error>),

    #[error(display = "Request timed out")]
    TimeoutError,

    #[error(display = "Failed to deserialize data")]
    DeserializeError(#[error(source)] Arc<serde_json::Error>),

    /// Unexpected response code
    #[error(display = "Unexpected response status code {} - {}", _0, _1)]
    ApiError(StatusCode, String),

    /// The string given was not a valid URI.
    #[error(display = "Not a valid URI")]
    InvalidUri,

    #[error(display = "Set account token on factory with no access token store")]
    NoAccessTokenStore,
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        matches!(self, Error::HyperError(_) | Error::TimeoutError)
    }

    /// Return true if there was no route to the destination
    pub fn is_offline(&self) -> bool {
        match self {
            Error::HyperError(error) if error.is_connect() => {
                if let Some(cause) = error.source() {
                    if let Some(err) = cause.downcast_ref::<std::io::Error>() {
                        return err.raw_os_error() == Some(libc::ENETUNREACH);
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self, Error::Aborted)
    }

    /// Returns a new instance for which `abortable_stream::Aborted` is mapped to `Self::Aborted`.
    fn map_aborted(self) -> Self {
        if let Error::HyperError(error) = &self {
            let mut source = error.source();
            while let Some(error) = source {
                let io_error: Option<&std::io::Error> = error.downcast_ref();
                if let Some(io_error) = io_error {
                    let abort_error: Option<&crate::abortable_stream::Aborted> =
                        io_error.get_ref().and_then(|inner| inner.downcast_ref());
                    if abort_error.is_some() {
                        return Self::Aborted;
                    }
                }
                source = error.source();
            }
        }
        self
    }
}

use super::ApiEndpointUpdateCallback;

/// A service that executes HTTP requests, allowing for on-demand termination of all in-flight
/// requests
pub(crate) struct RequestService<
    T: Stream<Item = ApiConnectionMode>,
    F: ApiEndpointUpdateCallback + Send,
> {
    command_tx: Weak<mpsc::UnboundedSender<RequestCommand>>,
    command_rx: mpsc::UnboundedReceiver<RequestCommand>,
    connector_handle: HttpsConnectorWithSniHandle,
    client: hyper::Client<HttpsConnectorWithSni, hyper::Body>,
    proxy_config_provider: T,
    new_address_callback: F,
    address_cache: AddressCache,
    api_availability: ApiAvailabilityHandle,
}

impl<
        T: Stream<Item = ApiConnectionMode> + Unpin + Send + 'static,
        F: ApiEndpointUpdateCallback + Send + Sync + 'static,
    > RequestService<T, F>
{
    /// Constructs a new request service.
    pub async fn spawn(
        sni_hostname: Option<String>,
        api_availability: ApiAvailabilityHandle,
        address_cache: AddressCache,
        mut proxy_config_provider: T,
        new_address_callback: F,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> RequestServiceHandle {
        let (connector, connector_handle) = HttpsConnectorWithSni::new(
            sni_hostname,
            address_cache.clone(),
            #[cfg(target_os = "android")]
            socket_bypass_tx.clone(),
        );

        #[cfg(feature = "api-override")]
        let force_direct_connection = API.force_direct_connection;
        #[cfg(not(feature = "api-override"))]
        let force_direct_connection = false;

        if force_direct_connection {
            log::debug!("API proxies are disabled");
        } else if let Some(config) = proxy_config_provider.next().await {
            connector_handle.set_connection_mode(config);
        }

        let (command_tx, command_rx) = mpsc::unbounded();
        let client = Client::builder().build(connector);

        let command_tx = Arc::new(command_tx);

        let service = Self {
            command_tx: Arc::downgrade(&command_tx),
            command_rx,
            connector_handle,
            client,
            proxy_config_provider,
            new_address_callback,
            address_cache,
            api_availability,
        };
        let handle = RequestServiceHandle { tx: command_tx };
        tokio::spawn(service.into_future());
        handle
    }

    async fn process_command(&mut self, command: RequestCommand) {
        match command {
            RequestCommand::NewRequest(request, completion_tx) => {
                self.handle_new_request(request, completion_tx);
            }
            RequestCommand::Reset => {
                self.connector_handle.reset();
            }
            RequestCommand::NextApiConfig(completion_tx) => {
                #[cfg(feature = "api-override")]
                if API.force_direct_connection {
                    log::debug!("Ignoring API connection mode");
                    let _ = completion_tx.send(Ok(()));
                    return;
                }

                if let Some(new_config) = self.proxy_config_provider.next().await {
                    let endpoint = match new_config.get_endpoint() {
                        Some(endpoint) => endpoint,
                        None => self.address_cache.get_address().await,
                    };
                    // Switch to new connection mode unless rejected by address change callback
                    if (self.new_address_callback)(endpoint).await {
                        self.connector_handle.set_connection_mode(new_config);
                    }
                }

                let _ = completion_tx.send(Ok(()));
            }
        }
    }

    fn handle_new_request(
        &mut self,
        request: Request,
        completion_tx: oneshot::Sender<Result<Response>>,
    ) {
        let tx = self.command_tx.upgrade();

        let api_availability = self.api_availability.clone();
        let request_future = request.into_future(self.client.clone(), api_availability.clone());

        tokio::spawn(async move {
            let response = request_future.await.map_err(|error| error.map_aborted());

            // Switch API endpoint if the request failed due to a network error
            if let Err(err) = &response {
                if err.is_network_error() && !api_availability.get_state().is_offline() {
                    log::error!("{}", err.display_chain_with_msg("HTTP request failed"));
                    if let Some(tx) = tx {
                        let (completion_tx, _completion_rx) = oneshot::channel();
                        let _ = tx.unbounded_send(RequestCommand::NextApiConfig(completion_tx));
                    }
                }
            }

            if completion_tx.send(response).is_err() {
                log::trace!("Failed to send response to caller, caller channel is shut down");
            }
        });
    }

    async fn into_future(mut self) {
        while let Some(command) = self.command_rx.next().await {
            self.process_command(command).await;
        }
        self.connector_handle.reset();
    }
}

#[derive(Clone)]
/// A handle to interact with a spawned `RequestService`.
pub struct RequestServiceHandle {
    tx: Arc<mpsc::UnboundedSender<RequestCommand>>,
}

impl RequestServiceHandle {
    /// Resets the corresponding RequestService, dropping all in-flight requests.
    pub fn reset(&self) {
        let _ = self.tx.unbounded_send(RequestCommand::Reset);
    }

    /// Submits a `RestRequest` for execution to the request service.
    pub async fn request(&self, request: Request) -> Result<Response> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RequestCommand::NewRequest(request, completion_tx))
            .map_err(|_| Error::RestServiceDown)?;
        completion_rx.await.map_err(|_| Error::RestServiceDown)?
    }

    /// Forcibly update the connection mode.
    pub async fn next_api_endpoint(&self) -> Result<()> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RequestCommand::NextApiConfig(completion_tx))
            .map_err(|_| Error::RestServiceDown)?;
        completion_rx.await.map_err(|_| Error::RestServiceDown)?
    }
}

#[derive(Debug)]
pub(crate) enum RequestCommand {
    NewRequest(
        Request,
        oneshot::Sender<std::result::Result<Response, Error>>,
    ),
    Reset,
    NextApiConfig(oneshot::Sender<std::result::Result<(), Error>>),
}

/// A REST request that is sent to the RequestService to be executed.
#[derive(Debug)]
pub struct Request {
    request: hyper::Request<hyper::Body>,
    timeout: Duration,
    access_token_store: Option<AccessTokenStore>,
    account: Option<AccountToken>,
    expected_status: &'static [hyper::StatusCode],
}

impl Request {
    /// Constructs a GET request with the given URI. Returns an error if the URI is not valid.
    pub fn get(uri: &str) -> Result<Self> {
        let uri = hyper::Uri::from_str(uri).map_err(|_| Error::InvalidUri)?;

        let mut builder = http::request::Builder::new()
            .method(Method::GET)
            .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
            .header(header::ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(host) = uri.host() {
            builder = builder.header(
                header::HOST,
                HeaderValue::from_str(host).map_err(|_| Error::InvalidHeaderError)?,
            );
        };

        let request = builder.uri(uri).body(hyper::Body::empty())?;
        Ok(Self::new(request, None))
    }

    fn new(
        request: hyper::Request<hyper::Body>,
        access_token_store: Option<AccessTokenStore>,
    ) -> Self {
        Self {
            request,
            timeout: DEFAULT_TIMEOUT,
            access_token_store,
            account: None,
            expected_status: &[],
        }
    }

    /// Set the account token to obtain authentication for.
    /// This fails if no store is set.
    pub fn account(mut self, account: AccountToken) -> Result<Self> {
        if self.access_token_store.is_none() {
            return Err(Error::NoAccessTokenStore);
        }
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

    async fn into_future<C: Connect + Clone + Send + Sync + 'static>(
        self,
        hyper_client: hyper::Client<C>,
        api_availability: ApiAvailabilityHandle,
    ) -> Result<Response> {
        let timeout = self.timeout;
        let inner_fut = self.into_future_without_timeout(hyper_client, api_availability);
        tokio::time::timeout(timeout, inner_fut)
            .await
            .map_err(|_| Error::TimeoutError)?
    }

    async fn into_future_without_timeout<C: Connect + Clone + Send + Sync + 'static>(
        mut self,
        hyper_client: hyper::Client<C>,
        api_availability: ApiAvailabilityHandle,
    ) -> Result<Response> {
        let _ = api_availability.wait_for_unsuspend().await;

        // Obtain access token first
        if let (Some(account), Some(store)) = (&self.account, &self.access_token_store) {
            let access_token = store.get_token(account).await?;
            let auth = HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|_| Error::InvalidHeaderError)?;
            self.request
                .headers_mut()
                .insert(header::AUTHORIZATION, auth);
        }

        // Make request to hyper client
        let response = hyper_client
            .request(self.request)
            .await
            .map_err(Error::from);

        // Notify access token store of expired tokens
        if let (Some(account), Some(store)) = (&self.account, &self.access_token_store) {
            store.check_response(account, &response);
        }

        // Parse unexpected responses and errors

        let response = response?;

        if !self.expected_status.contains(&response.status()) {
            if !self.expected_status.is_empty() {
                log::error!(
                    "Unexpected HTTP status code {}, expected codes [{}]",
                    response.status(),
                    self.expected_status
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
            if !response.status().is_success() {
                return handle_error_response(response).await;
            }
        }

        Ok(Response::new(response))
    }

    /// Returns the URI of the request
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }
}

/// Successful result of a REST request
#[derive(Debug)]
pub struct Response {
    response: hyper::Response<hyper::Body>,
}

impl Response {
    fn new(response: hyper::Response<hyper::Body>) -> Self {
        Self { response }
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    pub fn headers(&self) -> &hyper::HeaderMap<HeaderValue> {
        self.response.headers()
    }

    pub async fn deserialize<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        let body_length = get_body_length(&self.response);
        deserialize_body_inner(self.response, body_length).await
    }
}

#[derive(serde::Deserialize)]
struct OldErrorResponse {
    pub code: String,
}

/// If `NewErrorResponse::type` is not defined it should default to "about:blank"
const DEFAULT_ERROR_TYPE: &str = "about:blank";
#[derive(serde::Deserialize)]
struct NewErrorResponse {
    pub r#type: Option<String>,
}

#[derive(Clone)]
pub struct RequestFactory {
    hostname: &'static str,
    token_store: Option<AccessTokenStore>,
    default_timeout: Duration,
}

impl RequestFactory {
    pub fn new(hostname: &'static str, token_store: Option<AccessTokenStore>) -> Self {
        Self {
            hostname,
            token_store,
            default_timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn request(&self, path: &str, method: Method) -> Result<Request> {
        Ok(
            Request::new(self.hyper_request(path, method)?, self.token_store.clone())
                .timeout(self.default_timeout),
        )
    }

    pub fn get(&self, path: &str) -> Result<Request> {
        self.request(path, Method::GET)
    }

    pub fn post(&self, path: &str) -> Result<Request> {
        self.request(path, Method::POST)
    }

    pub fn put(&self, path: &str) -> Result<Request> {
        self.request(path, Method::PUT)
    }

    pub fn delete(&self, path: &str) -> Result<Request> {
        self.request(path, Method::DELETE)
    }

    pub fn post_json<S: serde::Serialize>(&self, path: &str, body: &S) -> Result<Request> {
        self.json_request(Method::POST, path, body)
    }

    pub fn put_json<S: serde::Serialize>(&self, path: &str, body: &S) -> Result<Request> {
        self.json_request(Method::PUT, path, body)
    }

    pub fn default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    fn json_request<S: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &S,
    ) -> Result<Request> {
        let mut request = self.hyper_request(path, method)?;

        let json_body = serde_json::to_string(&body)?;
        let body_length = json_body.as_bytes().len() as u64;
        *request.body_mut() = json_body.into_bytes().into();

        let headers = request.headers_mut();
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&body_length.to_string())
                .map_err(|_| Error::InvalidHeaderError)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(Request::new(request, self.token_store.clone()).timeout(self.default_timeout))
    }

    fn hyper_request(&self, path: &str, method: Method) -> Result<hyper::Request<hyper::Body>> {
        let uri = self.get_uri(path)?;
        let request = http::request::Builder::new()
            .method(method)
            .uri(uri)
            .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
            .header(header::ACCEPT, HeaderValue::from_static("application/json"))
            .header(header::HOST, HeaderValue::from_static(self.hostname));

        let result = request.body(hyper::Body::empty())?;
        Ok(result)
    }

    fn get_uri(&self, path: &str) -> Result<Uri> {
        let uri = format!("https://{}/{}", self.hostname, path);
        hyper::Uri::from_str(&uri).map_err(|_| Error::InvalidUri)
    }
}

fn get_body_length(response: &hyper::Response<hyper::Body>) -> usize {
    response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|length| length.parse::<usize>().ok())
        .unwrap_or(0)
}

async fn handle_error_response<T>(response: hyper::Response<hyper::Body>) -> Result<T> {
    let status = response.status();
    let error_message = match status {
        hyper::StatusCode::METHOD_NOT_ALLOWED => "Method not allowed",
        status => match get_body_length(&response) {
            0 => status.canonical_reason().unwrap_or("Unexpected error"),
            body_length => {
                return match response.headers().get("content-type") {
                    Some(content_type) if content_type == "application/problem+json" => {
                        // TODO: We should make sure we unify the new error format and the old
                        // error format so that they both produce the same Errors for the same
                        // problems after being processed.
                        let err: NewErrorResponse =
                            deserialize_body_inner(response, body_length).await?;
                        // The new error type replaces the `code` field with the `type` field.
                        // This is what is used to programmatically check the error.
                        Err(Error::ApiError(
                            status,
                            err.r#type
                                .unwrap_or_else(|| String::from(DEFAULT_ERROR_TYPE)),
                        ))
                    }
                    _ => {
                        let err: OldErrorResponse =
                            deserialize_body_inner(response, body_length).await?;
                        Err(Error::ApiError(status, err.code))
                    }
                };
            }
        },
    };
    Err(Error::ApiError(status, error_message.to_owned()))
}

async fn deserialize_body_inner<T: serde::de::DeserializeOwned>(
    mut response: hyper::Response<hyper::Body>,
    body_length: usize,
) -> Result<T> {
    let mut body: Vec<u8> = Vec::with_capacity(body_length);
    while let Some(chunk) = response.body_mut().next().await {
        body.extend(&chunk?);
    }

    serde_json::from_slice(&body).map_err(Error::from)
}

#[derive(Clone)]
pub struct MullvadRestHandle {
    pub(crate) service: RequestServiceHandle,
    pub factory: RequestFactory,
    pub availability: ApiAvailabilityHandle,
}

impl MullvadRestHandle {
    pub(crate) fn new(
        service: RequestServiceHandle,
        factory: RequestFactory,
        address_cache: AddressCache,
        availability: ApiAvailabilityHandle,
    ) -> Self {
        let handle = Self {
            service,
            factory,
            availability,
        };
        #[cfg(feature = "api-override")]
        if API.disable_address_cache {
            return handle;
        }
        handle.spawn_api_address_fetcher(address_cache);
        handle
    }

    fn spawn_api_address_fetcher(&self, address_cache: AddressCache) {
        let handle = self.clone();
        let availability = self.availability.clone();

        tokio::spawn(async move {
            let api_proxy = crate::ApiProxy::new(handle);
            let mut next_delay = API_IP_CHECK_INITIAL;

            loop {
                talpid_time::sleep(next_delay).await;

                if let Err(error) = availability.wait_background().await {
                    log::error!("Failed while waiting for API: {}", error);
                    continue;
                }
                match api_proxy.clone().get_api_addrs().await {
                    Ok(new_addrs) => {
                        if let Some(addr) = new_addrs.get(0) {
                            log::debug!(
                                "Fetched new API address {:?}. Fetching again in {} hours",
                                addr,
                                API_IP_CHECK_INTERVAL.as_secs() / (60 * 60)
                            );
                            if let Err(err) = address_cache.set_address(*addr).await {
                                log::error!("Failed to save newly updated API address: {}", err);
                            }
                        } else {
                            log::error!("API returned no API addresses");
                        }

                        next_delay = API_IP_CHECK_INTERVAL;
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to fetch new API addresses: {}. Retrying in {} seconds",
                            err,
                            API_IP_CHECK_ERROR_INTERVAL.as_secs()
                        );

                        next_delay = API_IP_CHECK_ERROR_INTERVAL;
                    }
                }
            }
        });
    }

    pub fn service(&self) -> RequestServiceHandle {
        self.service.clone()
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
