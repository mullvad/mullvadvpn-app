#[cfg(target_os = "android")]
pub use crate::https_client_with_sni::SocketBypassRequest;
use crate::{
    access::AccessTokenProxy,
    address_cache::AddressCache,
    availability::ApiAvailabilityHandle,
    https_client_with_sni::{HttpsConnectorWithSni, HttpsConnectorWithSniHandle},
    proxy::ApiConnectionMode,
};
use futures::{
    channel::{mpsc, oneshot},
    stream::StreamExt,
    Stream, TryFutureExt,
};
use hyper::{
    client::Client,
    header::{self, HeaderValue},
    Method, Uri,
};
use mullvad_types::account::AccountToken;
use std::{
    future::Future,
    str::FromStr,
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_types::ErrorExt;

#[cfg(feature = "api-override")]
use crate::API;

pub use hyper::StatusCode;

pub type Request = hyper::Request<hyper::Body>;
pub type Response = hyper::Response<hyper::Body>;

const USER_AGENT: &str = "mullvad-app";

const API_IP_CHECK_INITIAL: Duration = Duration::from_secs(15 * 60);
const API_IP_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const API_IP_CHECK_ERROR_INTERVAL: Duration = Duration::from_secs(15 * 60);

pub type Result<T> = std::result::Result<T, Error>;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Describes all the ways a REST request can fail
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Request cancelled")]
    Aborted,

    #[error(display = "Hyper error")]
    HyperError(#[error(source)] hyper::Error),

    #[error(display = "Invalid header value")]
    InvalidHeaderError(#[error(source)] http::header::InvalidHeaderValue),

    #[error(display = "HTTP error")]
    HttpError(#[error(source)] http::Error),

    #[error(display = "Request timed out")]
    TimeoutError(#[error(source)] tokio::time::error::Elapsed),

    #[error(display = "Failed to deserialize data")]
    DeserializeError(#[error(source)] serde_json::Error),

    #[error(display = "Failed to send request to rest client")]
    SendError,

    #[error(display = "Failed to receive response from rest client")]
    ReceiveError,

    /// Unexpected response code
    #[error(display = "Unexpected response status code {} - {}", _0, _1)]
    ApiError(StatusCode, String),

    /// The string given was not a valid URI.
    #[error(display = "Not a valid URI")]
    UriError(#[error(source)] http::uri::InvalidUri),
}

impl Error {
    pub fn is_network_error(&self) -> bool {
        matches!(self, Error::HyperError(_) | Error::TimeoutError(_))
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self, Error::Aborted)
    }

    /// Returns a new instance for which `abortable_stream::Aborted` is mapped to `Self::Aborted`.
    fn map_aborted(self) -> Self {
        if let Error::HyperError(error) = &self {
            use std::error::Error;
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
                let tx = self.command_tx.upgrade();
                let timeout = request.timeout();

                let hyper_request = request.into_request();

                let api_availability = self.api_availability.clone();
                let suspend_fut = api_availability.wait_for_unsuspend();
                let request_fut = self.client.request(hyper_request).map_err(Error::from);

                let request_future = async move {
                    let _ = suspend_fut.await;
                    request_fut.await
                };

                let future = async move {
                    let response = tokio::time::timeout(timeout, request_future)
                        .await
                        .map_err(Error::TimeoutError);

                    let response = flatten_result(response).map_err(|error| error.map_aborted());

                    if let Err(err) = &response {
                        if err.is_network_error() && !api_availability.get_state().is_offline() {
                            log::error!("{}", err.display_chain_with_msg("HTTP request failed"));
                            if let Some(tx) = tx {
                                let _ = tx.unbounded_send(RequestCommand::NextApiConfig);
                            }
                        }
                    }

                    if completion_tx.send(response).is_err() {
                        log::trace!(
                            "Failed to send response to caller, caller channel is shut down"
                        );
                    }
                };
                tokio::spawn(future);
            }
            RequestCommand::Reset => {
                self.connector_handle.reset();
            }
            RequestCommand::NextApiConfig => {
                #[cfg(feature = "api-override")]
                if API.force_direct_connection {
                    log::debug!("Ignoring API connection mode");
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
            }
        }
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

    /// Submits a `RestRequest` for exectuion to the request service.
    pub async fn request(&self, request: RestRequest) -> Result<Response> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RequestCommand::NewRequest(request, completion_tx))
            .map_err(|_| Error::SendError)?;
        completion_rx.await.map_err(|_| Error::ReceiveError)?
    }

    /// Forcibly update the connection mode.
    pub fn next_api_endpoint(&self) -> Result<()> {
        self.tx
            .unbounded_send(RequestCommand::NextApiConfig)
            .map_err(|_| Error::SendError)
    }
}

#[derive(Debug)]
pub(crate) enum RequestCommand {
    NewRequest(
        RestRequest,
        oneshot::Sender<std::result::Result<Response, Error>>,
    ),
    Reset,
    NextApiConfig,
}

/// A REST request that is sent to the RequestService to be executed.
#[derive(Debug)]
pub struct RestRequest {
    request: Request,
    timeout: Duration,
    auth: Option<HeaderValue>,
}

impl RestRequest {
    /// Constructs a GET request with the given URI. Returns an error if the URI is not valid.
    pub fn get(uri: &str) -> Result<Self> {
        let uri = hyper::Uri::from_str(uri).map_err(Error::UriError)?;

        let mut builder = http::request::Builder::new()
            .method(Method::GET)
            .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
            .header(header::ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(host) = uri.host() {
            builder = builder.header(header::HOST, HeaderValue::from_str(host)?);
        };

        let request = builder
            .uri(uri)
            .body(hyper::Body::empty())
            .map_err(Error::HttpError)?;

        Ok(RestRequest {
            timeout: DEFAULT_TIMEOUT,
            auth: None,
            request,
        })
    }

    /// Set the auth header with the following format: `Bearer $auth`.
    pub fn set_auth(&mut self, auth: Option<String>) -> Result<()> {
        let header = match auth {
            Some(auth) => Some(
                HeaderValue::from_str(&format!("Bearer {auth}"))
                    .map_err(Error::InvalidHeaderError)?,
            ),
            None => None,
        };

        self.auth = header;
        Ok(())
    }

    /// Sets timeout for the request.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Retrieves timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn add_header<T: header::IntoHeaderName>(&mut self, key: T, value: &str) -> Result<()> {
        let header_value = http::HeaderValue::from_str(value).map_err(Error::InvalidHeaderError)?;
        self.request.headers_mut().insert(key, header_value);
        Ok(())
    }

    /// Converts into a `hyper::Request<hyper::Body>`
    fn into_request(self) -> Request {
        let Self {
            mut request, auth, ..
        } = self;
        if let Some(auth) = auth {
            request.headers_mut().insert(header::AUTHORIZATION, auth);
        }
        request
    }

    /// Returns the URI of the request
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }
}

impl From<Request> for RestRequest {
    fn from(request: Request) -> Self {
        Self {
            request,
            timeout: DEFAULT_TIMEOUT,
            auth: None,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ErrorResponse {
    pub code: String,
}

#[derive(Clone)]
pub struct RequestFactory {
    hostname: String,
    path_prefix: Option<String>,
    pub timeout: Duration,
}

impl RequestFactory {
    pub fn new(hostname: String, path_prefix: Option<String>) -> Self {
        Self {
            hostname,
            path_prefix,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn request(&self, path: &str, method: Method) -> Result<RestRequest> {
        self.hyper_request(path, method)
            .map(RestRequest::from)
            .map(|req| self.set_request_timeout(req))
    }

    pub fn get(&self, path: &str) -> Result<RestRequest> {
        self.hyper_request(path, Method::GET)
            .map(RestRequest::from)
            .map(|req| self.set_request_timeout(req))
    }

    pub fn post(&self, path: &str) -> Result<RestRequest> {
        self.hyper_request(path, Method::POST)
            .map(RestRequest::from)
            .map(|req| self.set_request_timeout(req))
    }

    pub fn post_json<S: serde::Serialize>(&self, path: &str, body: &S) -> Result<RestRequest> {
        self.json_request(Method::POST, path, body)
    }

    fn json_request<S: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &S,
    ) -> Result<RestRequest> {
        let mut request = self.hyper_request(path, method)?;

        let json_body = serde_json::to_string(&body)?;
        let body_length = json_body.as_bytes().len() as u64;
        *request.body_mut() = json_body.into_bytes().into();

        let headers = request.headers_mut();
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&body_length.to_string()).map_err(Error::InvalidHeaderError)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(self.set_request_timeout(RestRequest::from(request)))
    }

    pub fn delete(&self, path: &str) -> Result<RestRequest> {
        self.hyper_request(path, Method::DELETE)
            .map(RestRequest::from)
            .map(|req| self.set_request_timeout(req))
    }

    fn hyper_request(&self, path: &str, method: Method) -> Result<Request> {
        let uri = self.get_uri(path)?;
        let request = http::request::Builder::new()
            .method(method)
            .uri(uri)
            .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
            .header(header::ACCEPT, HeaderValue::from_static("application/json"))
            .header(header::HOST, self.hostname.clone());

        request.body(hyper::Body::empty()).map_err(Error::HttpError)
    }

    fn get_uri(&self, path: &str) -> Result<Uri> {
        let prefix = self.path_prefix.as_ref().map(AsRef::as_ref).unwrap_or("");
        let uri = format!("https://{}/{}{}", self.hostname, prefix, path);
        hyper::Uri::from_str(&uri).map_err(Error::UriError)
    }

    fn set_request_timeout(&self, mut request: RestRequest) -> RestRequest {
        request.timeout = self.timeout;
        request
    }
}

pub fn get_request<T: serde::de::DeserializeOwned>(
    factory: &RequestFactory,
    service: RequestServiceHandle,
    uri: &str,
    auth: Option<String>,
    expected_statuses: &'static [hyper::StatusCode],
) -> impl Future<Output = Result<Response>> + 'static {
    let request = factory.get(uri);
    async move {
        let mut request = request?;
        request.set_auth(auth)?;
        let response = service.request(request).await?;
        parse_rest_response(response, expected_statuses).await
    }
}

pub fn send_request(
    factory: &RequestFactory,
    service: RequestServiceHandle,
    uri: &str,
    method: Method,
    auth: Option<(AccessTokenProxy, AccountToken)>,
    expected_statuses: &'static [hyper::StatusCode],
) -> impl Future<Output = Result<Response>> {
    let request = factory.request(uri, method);

    async move {
        let mut request = request?;
        if let Some((store, account)) = &auth {
            let access_token = store.get_token(account).await?;
            request.set_auth(Some(access_token))?;
        }
        let response = service.request(request).await?;
        let result = parse_rest_response(response, expected_statuses).await;

        if let Some((store, account)) = &auth {
            store.check_response(account, &result);
        }

        result
    }
}

pub fn send_json_request<B: serde::Serialize>(
    factory: &RequestFactory,
    service: RequestServiceHandle,
    uri: &str,
    method: Method,
    body: &B,
    auth: Option<(AccessTokenProxy, AccountToken)>,
    expected_statuses: &'static [hyper::StatusCode],
) -> impl Future<Output = Result<Response>> {
    let request = factory.json_request(method, uri, body);
    async move {
        let mut request = request?;
        if let Some((store, account)) = &auth {
            let access_token = store.get_token(account).await?;
            request.set_auth(Some(access_token))?;
        }
        let response = service.request(request).await?;
        let result = parse_rest_response(response, expected_statuses).await;

        if let Some((store, account)) = &auth {
            store.check_response(account, &result);
        }

        result
    }
}

pub async fn deserialize_body<T: serde::de::DeserializeOwned>(response: Response) -> Result<T> {
    let body_length = get_body_length(&response);
    deserialize_body_inner(response, body_length).await
}

async fn deserialize_body_inner<T: serde::de::DeserializeOwned>(
    mut response: Response,
    body_length: usize,
) -> Result<T> {
    let mut body: Vec<u8> = Vec::with_capacity(body_length);
    while let Some(chunk) = response.body_mut().next().await {
        body.extend(&chunk?);
    }

    serde_json::from_slice(&body).map_err(Error::DeserializeError)
}

fn get_body_length(response: &Response) -> usize {
    response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|length| length.parse::<usize>().ok())
        .unwrap_or(0)
}

pub async fn parse_rest_response(
    response: Response,
    expected_statuses: &'static [hyper::StatusCode],
) -> Result<Response> {
    if !expected_statuses.contains(&response.status()) {
        log::error!(
            "Unexpected HTTP status code {}, expected codes [{}]",
            response.status(),
            expected_statuses
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        if !response.status().is_success() {
            return handle_error_response(response).await;
        }
    }

    Ok(response)
}

pub async fn handle_error_response<T>(response: Response) -> Result<T> {
    let status = response.status();
    let error_message = match status {
        hyper::StatusCode::METHOD_NOT_ALLOWED => "Method not allowed",
        status => match get_body_length(&response) {
            0 => status.canonical_reason().unwrap_or("Unexpected error"),
            body_length => {
                let err: ErrorResponse = deserialize_body_inner(response, body_length).await?;
                return Err(Error::ApiError(status, err.code));
            }
        },
    };
    Err(Error::ApiError(status, error_message.to_owned()))
}

#[derive(Clone)]
pub struct MullvadRestHandle {
    pub(crate) service: RequestServiceHandle,
    pub factory: RequestFactory,
    pub availability: ApiAvailabilityHandle,
    pub token_store: AccessTokenProxy,
}

impl MullvadRestHandle {
    pub(crate) fn new(
        service: RequestServiceHandle,
        factory: RequestFactory,
        address_cache: AddressCache,
        availability: ApiAvailabilityHandle,
    ) -> Self {
        let token_store = AccessTokenProxy::new(service.clone(), factory.clone());

        let handle = Self {
            service,
            factory,
            availability,
            token_store,
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

    pub fn factory(&self) -> &RequestFactory {
        &self.factory
    }
}

fn flatten_result<T, E>(
    result: std::result::Result<std::result::Result<T, E>, E>,
) -> std::result::Result<T, E> {
    match result {
        Ok(value) => value,
        Err(err) => Err(err),
    }
}
