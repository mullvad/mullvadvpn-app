#![deny(rust_2018_idioms)]

use chrono::{offset::Utc, DateTime};
#[cfg(target_os = "android")]
use futures::channel::mpsc;
use futures::Stream;
use hyper::Method;
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    version::AppVersion,
};
use once_cell::sync::OnceCell;
use proxy::ApiConnectionMode;
use std::{
    cell::Cell,
    collections::BTreeMap,
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
    path::Path,
};
use talpid_types::ErrorExt;

pub mod availability;
use availability::{ApiAvailability, ApiAvailabilityHandle};
pub mod rest;

mod abortable_stream;
mod https_client_with_sni;
pub mod proxy;
mod tls_stream;
#[cfg(target_os = "android")]
pub use crate::https_client_with_sni::SocketBypassRequest;

mod access;
mod address_cache;
pub mod device;
mod fs;
mod relay_list;
pub use address_cache::AddressCache;
pub use device::DevicesProxy;
pub use hyper::StatusCode;
pub use relay_list::RelayListProxy;

/// Error code returned by the Mullvad API if the voucher has alreaby been used.
pub const VOUCHER_USED: &str = "VOUCHER_USED";

/// Error code returned by the Mullvad API if the voucher code is invalid.
pub const INVALID_VOUCHER: &str = "INVALID_VOUCHER";

/// Error code returned by the Mullvad API if the account token is invalid.
pub const INVALID_ACCOUNT: &str = "INVALID_ACCOUNT";

/// Error code returned by the Mullvad API if the device does not exist.
pub const DEVICE_NOT_FOUND: &str = "DEVICE_NOT_FOUND";

/// Error code returned by the Mullvad API if the access token is invalid.
pub const INVALID_ACCESS_TOKEN: &str = "INVALID_ACCESS_TOKEN";

pub const MAX_DEVICES_REACHED: &str = "MAX_DEVICES_REACHED";
pub const PUBKEY_IN_USE: &str = "PUBKEY_IN_USE";

pub const API_IP_CACHE_FILENAME: &str = "api-ip-address.txt";

const ACCOUNTS_URL_PREFIX: &str = "accounts/v1";
const APP_URL_PREFIX: &str = "app/v1";

pub static API: LazyManual<ApiEndpoint> = LazyManual::new(ApiEndpoint::from_env_vars);

unsafe impl<T, F: Send> Sync for LazyManual<T, F> where OnceCell<T>: Sync {}

/// A value that is either initialized on access or explicitly.
pub struct LazyManual<T, F = fn() -> T> {
    cell: OnceCell<T>,
    lazy_fn: Cell<Option<F>>,
}

impl<T, F> LazyManual<T, F> {
    const fn new(lazy_fn: F) -> Self {
        Self {
            cell: OnceCell::new(),
            lazy_fn: Cell::new(Some(lazy_fn)),
        }
    }

    /// Tries to initialize the object. An error is returned if it is
    /// already initialized.
    #[cfg(feature = "api-override")]
    pub fn override_init(&self, val: T) -> Result<(), T> {
        let _ = self.lazy_fn.take();
        self.cell.set(val)
    }
}

impl<T> Deref for LazyManual<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cell.get_or_init(|| (self.lazy_fn.take().unwrap())())
    }
}

/// A hostname and socketaddr to reach the Mullvad REST API over.
#[derive(Debug)]
pub struct ApiEndpoint {
    pub host: String,
    pub addr: SocketAddr,
    #[cfg(feature = "api-override")]
    pub disable_address_cache: bool,
    #[cfg(feature = "api-override")]
    pub disable_tls: bool,
    #[cfg(feature = "api-override")]
    pub force_direct_connection: bool,
}

impl ApiEndpoint {
    /// Returns the endpoint to connect to the API over.
    ///
    /// # Panics
    ///
    /// Panics if `MULLVAD_API_ADDR` has invalid contents or if only one of
    /// `MULLVAD_API_ADDR` or `MULLVAD_API_HOST` has been set but not the other.
    pub fn from_env_vars() -> ApiEndpoint {
        const API_HOST_DEFAULT: &str = "api.mullvad.net";
        const API_IP_DEFAULT: IpAddr = IpAddr::V4(Ipv4Addr::new(45, 83, 223, 196));
        const API_PORT_DEFAULT: u16 = 443;

        fn read_var(key: &'static str) -> Option<String> {
            use std::env;
            match env::var(key) {
                Ok(v) => Some(v),
                Err(env::VarError::NotPresent) => None,
                Err(env::VarError::NotUnicode(_)) => panic!("{key} does not contain valid UTF-8"),
            }
        }

        let host_var = read_var("MULLVAD_API_HOST");
        let address_var = read_var("MULLVAD_API_ADDR");
        let disable_tls_var = read_var("MULLVAD_API_DISABLE_TLS");

        #[cfg_attr(not(feature = "api-override"), allow(unused_mut))]
        let mut api = ApiEndpoint {
            host: API_HOST_DEFAULT.to_owned(),
            addr: SocketAddr::new(API_IP_DEFAULT, API_PORT_DEFAULT),
            #[cfg(feature = "api-override")]
            disable_address_cache: false,
            #[cfg(feature = "api-override")]
            disable_tls: false,
            #[cfg(feature = "api-override")]
            force_direct_connection: false,
        };

        #[cfg(feature = "api-override")]
        {
            use std::net::ToSocketAddrs;

            if host_var.is_none() && address_var.is_none() {
                if disable_tls_var.is_some() {
                    log::warn!("MULLVAD_API_DISABLE_TLS is ignored since MULLVAD_API_HOST and MULLVAD_API_ADDR are not set");
                }
                return api;
            }

            let scheme = if let Some(disable_tls_var) = disable_tls_var {
                api.disable_tls = disable_tls_var != "0";
                "http://"
            } else {
                "https://"
            };

            if let Some(user_host) = host_var {
                api.host = user_host;
            }
            if let Some(user_addr) = address_var {
                api.addr = user_addr
                    .parse()
                    .expect("MULLVAD_API_ADDR is not a valid socketaddr");
            } else {
                log::warn!("Resolving API IP from MULLVAD_API_HOST");
                api.addr = format!("{}:{}", api.host, API_PORT_DEFAULT)
                    .to_socket_addrs()
                    .expect("failed to resolve API host")
                    .next()
                    .expect("API host yielded 0 addresses");
            }
            api.disable_address_cache = true;
            api.force_direct_connection = true;
            log::debug!("Overriding API. Using {} at {scheme}{}", api.host, api.addr);
        }
        #[cfg(not(feature = "api-override"))]
        if host_var.is_some() || address_var.is_some() || disable_tls_var.is_some() {
            log::warn!("These variables are ignored in production builds: MULLVAD_API_HOST, MULLVAD_API_ADDR, MULLVAD_API_DISABLE_TLS");
        }
        api
    }
}

/// A type that helps with the creation of API connections.
pub struct Runtime {
    handle: tokio::runtime::Handle,
    pub address_cache: AddressCache,
    api_availability: availability::ApiAvailability,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to construct a rest client")]
    RestError(#[error(source)] rest::Error),

    #[error(display = "Failed to load address cache")]
    AddressCacheError(#[error(source)] address_cache::Error),

    #[error(display = "API availability check failed")]
    ApiCheckError(#[error(source)] availability::Error),
}

/// Closure that receives the next API (real or proxy) endpoint to use for `api.mullvad.net`.
/// It should return a future that determines whether to reject the new endpoint or not.
pub trait ApiEndpointUpdateCallback: Fn(SocketAddr) -> Self::AcceptedNewEndpoint {
    type AcceptedNewEndpoint: Future<Output = bool> + Send;
}

impl<U, T: Future<Output = bool> + Send> ApiEndpointUpdateCallback for U
where
    U: Fn(SocketAddr) -> T,
{
    type AcceptedNewEndpoint = T;
}

impl Runtime {
    /// Create a new `Runtime`.
    pub fn new(handle: tokio::runtime::Handle) -> Result<Self, Error> {
        Self::new_inner(
            handle,
            #[cfg(target_os = "android")]
            None,
        )
    }

    fn new_inner(
        handle: tokio::runtime::Handle,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> Result<Self, Error> {
        Ok(Runtime {
            handle,
            address_cache: AddressCache::new(None)?,
            api_availability: ApiAvailability::new(availability::State::default()),
            #[cfg(target_os = "android")]
            socket_bypass_tx,
        })
    }

    /// Create a new `Runtime` using the specified directories.
    /// Try to use the cache directory first, and fall back on the bundled address otherwise.
    pub async fn with_cache(
        cache_dir: &Path,
        write_changes: bool,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> Result<Self, Error> {
        let handle = tokio::runtime::Handle::current();
        #[cfg(feature = "api-override")]
        if API.disable_address_cache {
            return Self::new_inner(
                handle,
                #[cfg(target_os = "android")]
                socket_bypass_tx,
            );
        }

        let cache_file = cache_dir.join(API_IP_CACHE_FILENAME);
        let write_file = if write_changes {
            Some(cache_file.clone().into_boxed_path())
        } else {
            None
        };

        let address_cache = match AddressCache::from_file(&cache_file, write_file.clone()).await {
            Ok(cache) => cache,
            Err(error) => {
                if cache_file.exists() {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to load cached API addresses. Falling back on bundled address"
                        )
                    );
                }
                AddressCache::new(write_file)?
            }
        };

        Ok(Runtime {
            handle,
            address_cache,
            api_availability: ApiAvailability::new(availability::State::default()),
            #[cfg(target_os = "android")]
            socket_bypass_tx,
        })
    }

    /// Creates a new request service and returns a handle to it.
    async fn new_request_service<T: Stream<Item = ApiConnectionMode> + Unpin + Send + 'static>(
        &self,
        sni_hostname: Option<String>,
        proxy_provider: T,
        new_address_callback: impl ApiEndpointUpdateCallback + Send + Sync + 'static,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> rest::RequestServiceHandle {
        rest::RequestService::spawn(
            sni_hostname,
            self.api_availability.handle(),
            self.address_cache.clone(),
            proxy_provider,
            new_address_callback,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
        )
        .await
    }

    /// Returns a request factory initialized to create requests for the master API
    pub async fn mullvad_rest_handle<
        T: Stream<Item = ApiConnectionMode> + Unpin + Send + 'static,
    >(
        &self,
        proxy_provider: T,
        new_address_callback: impl ApiEndpointUpdateCallback + Send + Sync + 'static,
    ) -> rest::MullvadRestHandle {
        let service = self
            .new_request_service(
                Some(API.host.clone()),
                proxy_provider,
                new_address_callback,
                #[cfg(target_os = "android")]
                self.socket_bypass_tx.clone(),
            )
            .await;
        let factory = rest::RequestFactory::new(API.host.clone(), None);

        rest::MullvadRestHandle::new(
            service,
            factory,
            self.address_cache.clone(),
            self.availability_handle(),
        )
    }

    /// Returns a new request service handle
    pub async fn rest_handle(&mut self) -> rest::RequestServiceHandle {
        self.new_request_service(
            None,
            ApiConnectionMode::Direct.into_repeat(),
            |_| async { true },
            #[cfg(target_os = "android")]
            None,
        )
        .await
    }

    pub fn handle(&mut self) -> &mut tokio::runtime::Handle {
        &mut self.handle
    }

    pub fn availability_handle(&self) -> ApiAvailabilityHandle {
        self.api_availability.handle()
    }
}

#[derive(Clone)]
pub struct AccountsProxy {
    handle: rest::MullvadRestHandle,
}

impl AccountsProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn get_expiry(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<DateTime<Utc>, rest::Error>> {
        #[derive(serde::Deserialize)]
        struct AccountExpiryResponse {
            expiry: DateTime<Utc>,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let access_proxy = self.handle.token_store.clone();
        async move {
            let response = rest::send_request(
                &factory,
                service,
                &format!("{ACCOUNTS_URL_PREFIX}/accounts/me"),
                Method::GET,
                Some((access_proxy, account)),
                &[StatusCode::OK],
            )
            .await;

            let account: AccountExpiryResponse = rest::deserialize_body(response?).await?;
            Ok(account.expiry)
        }
    }

    pub fn create_account(&mut self) -> impl Future<Output = Result<AccountToken, rest::Error>> {
        #[derive(serde::Deserialize)]
        struct AccountCreationResponse {
            number: AccountToken,
        }

        let service = self.handle.service.clone();
        let response = rest::send_request(
            &self.handle.factory,
            service,
            &format!("{ACCOUNTS_URL_PREFIX}/accounts"),
            Method::POST,
            None,
            &[StatusCode::CREATED],
        );

        async move {
            let account: AccountCreationResponse = rest::deserialize_body(response.await?).await?;
            Ok(account.number)
        }
    }

    pub fn submit_voucher(
        &mut self,
        account_token: AccountToken,
        voucher_code: String,
    ) -> impl Future<Output = Result<VoucherSubmission, rest::Error>> {
        #[derive(serde::Serialize)]
        struct VoucherSubmission {
            voucher_code: String,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let access_proxy = self.handle.token_store.clone();
        let submission = VoucherSubmission { voucher_code };

        async move {
            let response = rest::send_json_request(
                &factory,
                service,
                &format!("{APP_URL_PREFIX}/submit-voucher"),
                Method::POST,
                &submission,
                Some((access_proxy, account_token)),
                &[StatusCode::OK],
            )
            .await;
            rest::deserialize_body(response?).await
        }
    }

    pub fn get_www_auth_token(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<String, rest::Error>> {
        #[derive(serde::Deserialize)]
        struct AuthTokenResponse {
            auth_token: String,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let access_proxy = self.handle.token_store.clone();

        async move {
            let response = rest::send_request(
                &factory,
                service,
                &format!("{APP_URL_PREFIX}/www-auth-token"),
                Method::POST,
                Some((access_proxy, account)),
                &[StatusCode::OK],
            )
            .await;
            let response: AuthTokenResponse = rest::deserialize_body(response?).await?;
            Ok(response.auth_token)
        }
    }
}

pub struct ProblemReportProxy {
    handle: rest::MullvadRestHandle,
}

impl ProblemReportProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn problem_report(
        &self,
        email: &str,
        message: &str,
        log: &str,
        metadata: &BTreeMap<String, String>,
    ) -> impl Future<Output = Result<(), rest::Error>> {
        #[derive(serde::Serialize)]
        struct ProblemReport {
            address: String,
            message: String,
            log: String,
            metadata: BTreeMap<String, String>,
        }

        let report = ProblemReport {
            address: email.to_owned(),
            message: message.to_owned(),
            log: log.to_owned(),
            metadata: metadata.clone(),
        };

        let service = self.handle.service.clone();

        let request = rest::send_json_request(
            &self.handle.factory,
            service,
            &format!("{APP_URL_PREFIX}/problem-report"),
            Method::POST,
            &report,
            None,
            &[StatusCode::NO_CONTENT],
        );

        async move {
            request.await?;
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct AppVersionProxy {
    handle: rest::MullvadRestHandle,
}

#[derive(serde::Deserialize, Debug)]
pub struct AppVersionResponse {
    pub supported: bool,
    pub latest: AppVersion,
    pub latest_stable: Option<AppVersion>,
    pub latest_beta: AppVersion,
    #[serde(default = "default_wg_threshold")]
    pub x_threshold_wg_default: f32,
}

/// Temporary function that will be removed later. Used to generate default wg_threshold.
/// In case there is no `x_threshold_wg_default` returned by the API result we interpret that to
/// mean that the migration is done and WireGuard should be the default. In that case the threshold
/// value should be 1.0
fn default_wg_threshold() -> f32 {
    1.0
}

impl AppVersionProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn version_check(
        &self,
        app_version: AppVersion,
        platform: &str,
        platform_version: String,
    ) -> impl Future<Output = Result<AppVersionResponse, rest::Error>> {
        let service = self.handle.service.clone();

        let path = format!("{APP_URL_PREFIX}/releases/{platform}/{app_version}");
        let request = self.handle.factory.request(&path, Method::GET);

        async move {
            let mut request = request?;
            request.add_header("M-Platform-Version", &platform_version)?;

            let response = service.request(request).await?;
            let parsed_response = rest::parse_rest_response(response, &[StatusCode::OK]).await?;
            rest::deserialize_body(parsed_response).await
        }
    }
}

#[derive(Clone)]
pub struct ApiProxy {
    handle: rest::MullvadRestHandle,
}

impl ApiProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub async fn get_api_addrs(&self) -> Result<Vec<SocketAddr>, rest::Error> {
        let service = self.handle.service.clone();

        let response = rest::send_request(
            &self.handle.factory,
            service,
            &format!("{APP_URL_PREFIX}/api-addrs"),
            Method::GET,
            None,
            &[StatusCode::OK],
        )
        .await?;

        rest::deserialize_body(response).await
    }
}
