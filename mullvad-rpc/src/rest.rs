use futures::{
    channel::{mpsc, oneshot},
    future::{abortable, AbortHandle, Aborted},
    sink::SinkExt,
    stream::StreamExt,
    TryFutureExt,
};
use hyper::{
    client::{connect::Connect, Client},
    header::{self, HeaderValue},
    Method, Uri,
};
use std::{collections::BTreeMap, future::Future, mem, net::IpAddr, str::FromStr, time::Duration};
use tokio::runtime::Handle;

pub use hyper::StatusCode;

pub type Request = hyper::Request<hyper::Body>;
pub type Response = hyper::Response<hyper::Body>;


pub type Result<T> = std::result::Result<T, Error>;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Describes all the ways a REST request can fail
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Request cancelled")]
    Aborted(Aborted),

    #[error(display = "Hyper error")]
    HyperError(#[error(source)] hyper::Error),

    #[error(display = "Invalid header value")]
    InvalidHeaderError(#[error(source)] http::header::InvalidHeaderValue),

    #[error(display = "HTTP error")]
    HttpError(#[error(source)] http::Error),

    #[error(display = "Request timed out")]
    TimeoutError(#[error(source)] tokio::time::Elapsed),

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

/// A service that executes HTTP requests, allowing for on-demand termination of all in-flight
/// requests
pub(crate) struct RequestService<C> {
    command_tx: mpsc::Sender<RequestCommand>,
    command_rx: mpsc::Receiver<RequestCommand>,
    client: hyper::Client<C, hyper::Body>,
    connector: C,
    handle: Handle,
    next_id: u64,
    in_flight_requests: BTreeMap<u64, AbortHandle>,
}

impl<C: Connect + Clone + Send + Sync + 'static> RequestService<C> {
    /// Constructs a new request service.
    pub fn new(connector: C, handle: Handle) -> RequestService<C> {
        let client = Self::new_client(connector.clone());

        let (command_tx, command_rx) = mpsc::channel(1);
        Self {
            command_tx,
            command_rx,
            client,
            in_flight_requests: BTreeMap::new(),
            next_id: 0,
            connector,
            handle,
        }
    }

    /// Constructs a handle
    pub fn handle(&self) -> RequestServiceHandle {
        RequestServiceHandle {
            tx: self.command_tx.clone(),
            handle: self.handle.clone(),
        }
    }

    fn new_client(connector: C) -> Client<C, hyper::Body> {
        Client::builder().pool_max_idle_per_host(0).build(connector)
    }

    fn process_command(&mut self, command: RequestCommand) {
        match command {
            RequestCommand::NewRequest(request, completion_tx) => {
                let id = self.id();
                let mut tx = self.command_tx.clone();
                let timeout = request.timeout();

                let (request_future, abort_handle) = abortable(
                    self.client
                        .request(request.into_request())
                        .map_err(Error::from),
                );

                let future = async move {
                    let response =
                        tokio::time::timeout(timeout, request_future.map_err(Error::Aborted))
                            .await
                            .map_err(Error::TimeoutError);

                    let response = flatten_result(flatten_result(response));

                    if completion_tx.send(response).is_err() {
                        log::trace!(
                            "Failed to send response to caller, caller channel is shut down"
                        );
                    }
                    let _ = tx.send(RequestCommand::RequestFinished(id)).await;
                };


                self.handle.spawn(future);
                self.in_flight_requests.insert(id, abort_handle);
            }

            RequestCommand::RequestFinished(id) => {
                self.in_flight_requests.remove(&id);
            }

            RequestCommand::Reset => {
                self.reset();
            }
        }
    }

    fn reset(&mut self) {
        let old_requests = mem::replace(&mut self.in_flight_requests, BTreeMap::new());
        for (_, abort_handle) in old_requests.into_iter() {
            abort_handle.abort();
        }
        let _ = mem::replace(&mut self.client, Self::new_client(self.connector.clone()));
        self.next_id = 0;
    }

    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = id.wrapping_add(1);
        id
    }

    pub async fn into_future(mut self) {
        while let Some(command) = self.command_rx.next().await {
            self.process_command(command);
        }
    }
}


#[derive(Clone)]
/// A handle to interact with a spawned `RequestService`.
pub struct RequestServiceHandle {
    tx: mpsc::Sender<RequestCommand>,
    handle: Handle,
}

impl RequestServiceHandle {
    /// Resets the corresponding RequestService, dropping all in-flight requests.
    pub async fn reset(&self) {
        let mut tx = self.tx.clone();

        let _ = tx.send(RequestCommand::Reset).await;
    }

    /// Submits a `RestRequest` for exectuion to the request service.
    pub async fn request(&self, request: RestRequest) -> Result<Response> {
        let (completion_tx, completion_rx) = oneshot::channel();
        let mut tx = self.tx.clone();
        tx.send(RequestCommand::NewRequest(request, completion_tx))
            .await
            .map_err(|_| Error::SendError)?;


        completion_rx.await.map_err(|_| Error::ReceiveError)?
    }

    /// Spawns a future on the RPC runtime.
    pub fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) {
        let _ = self.handle.spawn(future);
    }
}

#[derive(Debug)]
enum RequestCommand {
    NewRequest(
        RestRequest,
        oneshot::Sender<std::result::Result<Response, Error>>,
    ),
    RequestFinished(u64),
    Reset,
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
        let uri = hyper::Uri::from_str(&uri).map_err(Error::UriError)?;

        let mut builder = http::request::Builder::new()
            .method(Method::GET)
            .header(header::ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(host) = uri.host() {
            builder = builder.header(header::HOST, HeaderValue::from_str(&host)?);
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

    /// Set the auth header with the following format: `Token $auth`.
    pub fn set_auth(&mut self, auth: Option<String>) -> Result<()> {
        let header = match auth {
            Some(auth) => Some(
                HeaderValue::from_str(&format!("Token {}", auth))
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
    host: String,
    address: Option<IpAddr>,
    path_prefix: Option<String>,
    pub timeout: Duration,
}


impl RequestFactory {
    pub fn new(host: String, address: Option<IpAddr>, path_prefix: Option<String>) -> Self {
        Self {
            host,
            address,
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
        let mut request = self.hyper_request(path, Method::POST)?;

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

        Ok(RestRequest::from(request))
    }

    pub fn delete(&self, path: &str) -> Result<RestRequest> {
        self.hyper_request(path, Method::DELETE)
            .map(RestRequest::from)
    }

    fn hyper_request(&self, path: &str, method: Method) -> Result<Request> {
        let uri = self.get_uri(path)?;
        let request = http::request::Builder::new()
            .method(method)
            .uri(uri)
            .header(header::ACCEPT, HeaderValue::from_static("application/json"))
            .header(header::HOST, self.host.clone());

        request.body(hyper::Body::empty()).map_err(Error::HttpError)
    }

    fn get_uri(&self, path: &str) -> Result<Uri> {
        let host: &dyn std::fmt::Display = &self
            .address
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| self.host.clone());
        let prefix = self.path_prefix.as_ref().map(AsRef::as_ref).unwrap_or("");
        let uri = format!("https://{}/{}{}", host, prefix, path);
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
    expected_status: hyper::StatusCode,
) -> impl Future<Output = Result<Response>> {
    let request = factory.get(uri);
    async move {
        let mut request = request?;
        request.set_auth(auth)?;
        let response = service.request(request).await?;
        parse_rest_response(response, expected_status).await
    }
}

pub fn send_request(
    factory: &RequestFactory,
    service: RequestServiceHandle,
    uri: &str,
    method: Method,
    auth: Option<String>,
    expected_status: hyper::StatusCode,
) -> impl Future<Output = Result<Response>> {
    let request = factory.request(uri, method);

    async move {
        let mut request = request?;
        request.set_auth(auth)?;
        let response = service.request(request).await?;
        parse_rest_response(response, expected_status).await
    }
}

pub fn post_request_with_json<B: serde::Serialize>(
    factory: &RequestFactory,
    service: RequestServiceHandle,
    uri: &str,
    body: &B,
    auth: Option<String>,
    expected_status: hyper::StatusCode,
) -> impl Future<Output = Result<Response>> {
    let request = factory.post_json(uri, body);
    async move {
        let mut request = request?;
        request.set_auth(auth)?;
        let response = service.request(request).await?;
        parse_rest_response(response, expected_status).await
    }
}


pub async fn deserialize_body<T: serde::de::DeserializeOwned>(mut response: Response) -> Result<T> {
    let body_length: usize = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|length| length.parse::<usize>().ok())
        .unwrap_or(0);

    let mut body: Vec<u8> = Vec::with_capacity(body_length);
    while let Some(chunk) = response.body_mut().next().await {
        body.extend(&chunk?);
    }

    serde_json::from_slice(&body).map_err(Error::DeserializeError)
}

pub async fn parse_rest_response(
    response: Response,
    expected_status: hyper::StatusCode,
) -> Result<Response> {
    let status = response.status();
    if status != expected_status {
        return handle_error_response(response).await;
    }

    Ok(response)
}


pub async fn handle_error_response<T>(response: Response) -> Result<T> {
    let error_message = match response.status() {
        hyper::StatusCode::NOT_FOUND => "Not found",
        hyper::StatusCode::METHOD_NOT_ALLOWED => "Method not allowed",
        status => {
            let err: ErrorResponse = deserialize_body(response).await?;

            return Err(Error::ApiError(status, err.code));
        }
    };
    Err(Error::ApiError(response.status(), error_message.to_owned()))
}

#[derive(Clone)]
pub struct MullvadRestHandle {
    pub(crate) service: RequestServiceHandle,
    pub factory: RequestFactory,
}

impl MullvadRestHandle {
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
