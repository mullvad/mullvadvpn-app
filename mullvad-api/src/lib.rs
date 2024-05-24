#![allow(rustdoc::private_intra_doc_links)]
#[cfg(target_os = "android")]
use futures::channel::mpsc;
use hyper::Method;
#[cfg(target_os = "android")]
use mullvad_types::account::{PlayPurchase, PlayPurchasePaymentToken};
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    version::AppVersion,
};
use proxy::{ApiConnectionMode, ConnectionModeProvider};
use std::{
    cell::Cell,
    collections::BTreeMap,
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
    path::Path,
    sync::OnceLock,
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
mod relay_list;

#[cfg(target_os = "ios")]
pub mod ffi;

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
#[cfg(target_os = "android")]
const GOOGLE_PAYMENTS_URL_PREFIX: &str = "payments/google-play/v1";

pub static API: LazyManual<ApiEndpoint> = LazyManual::new(ApiEndpoint::from_env_vars);

unsafe impl<T, F: Send> Sync for LazyManual<T, F> where OnceLock<T>: Sync {}

/// A value that is either initialized on access or explicitly.
pub struct LazyManual<T, F = fn() -> T> {
    cell: OnceLock<T>,
    lazy_fn: Cell<Option<F>>,
}

impl<T, F> LazyManual<T, F> {
    const fn new(lazy_fn: F) -> Self {
        Self {
            cell: OnceLock::new(),
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

pub mod env {
    pub const API_HOST_VAR: &str = "MULLVAD_API_HOST";
    pub const API_ADDR_VAR: &str = "MULLVAD_API_ADDR";
    pub const API_FORCE_DIRECT_VAR: &str = "MULLVAD_API_FORCE_DIRECT";
    pub const DISABLE_TLS_VAR: &str = "MULLVAD_API_DISABLE_TLS";
}

/// A hostname and socketaddr to reach the Mullvad REST API over.
#[derive(Debug)]
pub struct ApiEndpoint {
    /// An overriden API hostname. Initialized with the value of the environment
    /// variable `MULLVAD_API_HOST` if it has been set.
    ///
    /// Use the associated function [`Self::host`] to read this value with a
    /// default fallback if `MULLVAD_API_HOST` was not set.
    pub host: Option<String>,
    /// An overriden API address. Initialized with the value of the environment
    /// variable `MULLVAD_API_ADDR` if it has been set.
    ///
    /// Use the associated function [`Self::address()`] to read this value with
    /// a default fallback if `MULLVAD_API_ADDR` was not set.
    ///
    /// # Note
    ///
    /// If [`Self::address`] is populated with [`Some(SocketAddr)`], it should
    /// always be respected when establishing API connections.
    pub address: Option<SocketAddr>,
    #[cfg(feature = "api-override")]
    pub disable_address_cache: bool,
    #[cfg(feature = "api-override")]
    pub disable_tls: bool,
    #[cfg(feature = "api-override")]
    /// Whether bridges/proxies can be used to access the API or not. This is
    /// useful primarily for testing purposes.
    ///
    /// * If `force_direct` is `true`, bridges and proxies will not be used to
    /// reach the API.
    /// * If `force_direct` is `false`, bridges and proxies can be used to reach the API.
    ///
    /// # Note
    ///
    /// By default, `force_direct` will be `true` if the `api-override` feature
    /// is enabled and overrides are in use. This is supposedly less error prone, as
    /// common targets such as Devmole might be unreachable from behind a bridge server.
    ///
    /// To disable `force_direct`, set the environment variable
    /// `MULLVAD_API_FORCE_DIRECT=0` before starting the daemon.
    pub force_direct: bool,
}

impl ApiEndpoint {
    const API_HOST_DEFAULT: &'static str = "api.mullvad.net";
    const API_IP_DEFAULT: IpAddr = IpAddr::V4(Ipv4Addr::new(45, 83, 223, 196));
    const API_PORT_DEFAULT: u16 = 443;

    /// Returns the endpoint to connect to the API over.
    ///
    /// # Panics
    ///
    /// Panics if `MULLVAD_API_ADDR`, `MULLVAD_API_HOST` or
    /// `MULLVAD_API_DISABLE_TLS` has invalid contents.
    #[cfg(feature = "api-override")]
    pub fn from_env_vars() -> ApiEndpoint {
        let host_var = Self::read_var(env::API_HOST_VAR);
        let address_var = Self::read_var(env::API_ADDR_VAR);
        let disable_tls_var = Self::read_var(env::DISABLE_TLS_VAR);
        let force_direct = Self::read_var(env::API_FORCE_DIRECT_VAR);

        let mut api = ApiEndpoint {
            host: None,
            address: None,
            disable_address_cache: host_var.is_some() || address_var.is_some(),
            disable_tls: false,
            force_direct: force_direct
                .map(|force_direct| force_direct != "0")
                .unwrap_or_else(|| host_var.is_some() || address_var.is_some()),
        };

        match (host_var, address_var) {
            (None, None) => {}
            (Some(host), None) => {
                use std::net::ToSocketAddrs;
                log::debug!(
                    "{api_addr} not found. Resolving API IP address from {api_host}={host}",
                    api_addr = env::API_ADDR_VAR,
                    api_host = env::API_HOST_VAR
                );
                api.address = format!("{}:{}", host, ApiEndpoint::API_PORT_DEFAULT)
                    .to_socket_addrs()
                    .unwrap_or_else(|_| {
                        panic!(
                            "Unable to resolve API IP address from host {host}:{port}",
                            port = ApiEndpoint::API_PORT_DEFAULT,
                        )
                    })
                    .next();
            }
            (host, Some(address)) => {
                let addr = address.parse().unwrap_or_else(|_| {
                    panic!(
                        "{api_addr}={address} is not a valid socketaddr",
                        api_addr = env::API_ADDR_VAR,
                    )
                });
                api.address = Some(addr);
                api.host = host;
            }
        }

        if api.host.is_none() && api.address.is_none() {
            if disable_tls_var.is_some() {
                log::warn!(
                    "{disable_tls} is ignored since {api_host} and {api_addr} are not set",
                    disable_tls = env::DISABLE_TLS_VAR,
                    api_host = env::API_HOST_VAR,
                    api_addr = env::API_ADDR_VAR,
                );
            }
        } else {
            api.disable_tls = disable_tls_var
                .as_ref()
                .map(|disable_tls| disable_tls != "0")
                .unwrap_or(api.disable_tls);

            log::debug!(
                "Overriding API. Using {host} at {scheme}{addr} (force direct={direct})",
                host = api.host(),
                addr = api.address(),
                scheme = if api.disable_tls {
                    "http://"
                } else {
                    "https://"
                },
                direct = api.force_direct,
            );
        }
        api
    }

    /// Returns the endpoint to connect to the API over.
    ///
    /// # Panics
    ///
    /// Panics if `MULLVAD_API_ADDR`, `MULLVAD_API_HOST` or
    /// `MULLVAD_API_DISABLE_TLS` has invalid contents.
    #[cfg(not(feature = "api-override"))]
    pub fn from_env_vars() -> ApiEndpoint {
        let env_vars = [
            env::API_HOST_VAR,
            env::API_ADDR_VAR,
            env::DISABLE_TLS_VAR,
            env::API_FORCE_DIRECT_VAR,
        ];

        if env_vars.map(Self::read_var).iter().any(Option::is_some) {
            log::warn!(
                "These variables are ignored in production builds: {env_vars_pretty}",
                env_vars_pretty = env_vars.join(", ")
            );
        }

        ApiEndpoint {
            host: None,
            address: None,
        }
    }

    /// Read the [`Self::host`] value, falling back to
    /// [`Self::API_HOST_DEFAULT`] as default value if it does not exist.
    pub fn host(&self) -> &str {
        self.host
            .as_deref()
            .unwrap_or(ApiEndpoint::API_HOST_DEFAULT)
    }

    /// Read the [`Self::address`] value, falling back to
    /// [`Self::API_IP_DEFAULT`] as default value if it does not exist.
    pub fn address(&self) -> SocketAddr {
        self.address.unwrap_or(SocketAddr::new(
            ApiEndpoint::API_IP_DEFAULT,
            ApiEndpoint::API_PORT_DEFAULT,
        ))
    }

    /// Try to read the value of an environment variable. Returns `None` if the
    /// environment variable has not been set.
    ///
    /// # Panics
    ///
    /// Panics if the environment variable was found, but it did not contain
    /// valid unicode data.
    fn read_var(key: &'static str) -> Option<String> {
        use std::env;
        match env::var(key) {
            Ok(v) => Some(v),
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(_)) => panic!("{key} does not contain valid UTF-8"),
        }
    }
}

/// A type that helps with the creation of API connections.
pub struct Runtime {
    handle: tokio::runtime::Handle,
    address_cache: AddressCache,
    api_availability: availability::ApiAvailability,
    #[cfg(target_os = "android")]
    socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to construct a rest client")]
    RestError(#[from] rest::Error),

    #[error("Failed to load address cache")]
    AddressCacheError(#[from] address_cache::Error),

    #[error("API availability check failed")]
    ApiCheckError(#[from] availability::Error),
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

    #[cfg(target_os = "ios")]
    pub fn with_static_addr(handle: tokio::runtime::Handle, address: SocketAddr) -> Self {
        Runtime {
            handle,
            address_cache: AddressCache::with_static_addr(address),
            api_availability: ApiAvailability::new(availability::State::default()),
        }
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
    fn new_request_service<T: ConnectionModeProvider + 'static>(
        &self,
        sni_hostname: Option<String>,
        connection_mode_provider: T,
        #[cfg(target_os = "android")] socket_bypass_tx: Option<mpsc::Sender<SocketBypassRequest>>,
    ) -> rest::RequestServiceHandle {
        rest::RequestService::spawn(
            sni_hostname,
            self.api_availability.handle(),
            self.address_cache.clone(),
            connection_mode_provider,
            #[cfg(target_os = "android")]
            socket_bypass_tx,
        )
    }

    /// Returns a request factory initialized to create requests for the master API
    pub fn mullvad_rest_handle<T: ConnectionModeProvider + 'static>(
        &self,
        connection_mode_provider: T,
    ) -> rest::MullvadRestHandle {
        let service = self.new_request_service(
            Some(API.host().to_string()),
            connection_mode_provider,
            #[cfg(target_os = "android")]
            self.socket_bypass_tx.clone(),
        );
        let token_store = access::AccessTokenStore::new(service.clone(), API.host());
        let factory = rest::RequestFactory::new(API.host().to_owned(), Some(token_store));

        rest::MullvadRestHandle::new(service, factory, self.availability_handle())
    }

    /// This is only to be used in test code
    pub fn static_mullvad_rest_handle(&self, hostname: String) -> rest::MullvadRestHandle {
        let service = self.new_request_service(
            Some(hostname.clone()),
            ApiConnectionMode::Direct.into_provider(),
            #[cfg(target_os = "android")]
            self.socket_bypass_tx.clone(),
        );
        let token_store = access::AccessTokenStore::new(service.clone(), hostname.clone());
        let factory = rest::RequestFactory::new(hostname, Some(token_store));

        rest::MullvadRestHandle::new(service, factory, self.availability_handle())
    }

    /// Returns a new request service handle
    pub fn rest_handle(&self) -> rest::RequestServiceHandle {
        self.new_request_service(
            None,
            ApiConnectionMode::Direct.into_provider(),
            #[cfg(target_os = "android")]
            None,
        )
    }

    pub fn handle(&mut self) -> &mut tokio::runtime::Handle {
        &mut self.handle
    }

    pub fn availability_handle(&self) -> ApiAvailabilityHandle {
        self.api_availability.handle()
    }

    pub fn address_cache(&self) -> &AddressCache {
        &self.address_cache
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

    pub fn get_data(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<AccountData, rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        async move {
            let request = factory
                .get(&format!("{ACCOUNTS_URL_PREFIX}/accounts/me"))?
                .expected_status(&[StatusCode::OK])
                .account(account)?;
            let response = service.request(request).await?;
            response.deserialize().await
        }
    }

    pub fn create_account(&self) -> impl Future<Output = Result<AccountToken, rest::Error>> {
        #[derive(serde::Deserialize)]
        struct AccountCreationResponse {
            number: AccountToken,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .post(&format!("{ACCOUNTS_URL_PREFIX}/accounts"))?
                .expected_status(&[StatusCode::CREATED]);
            let response = service.request(request).await?;
            let account: AccountCreationResponse = response.deserialize().await?;
            Ok(account.number)
        }
    }

    pub fn submit_voucher(
        &self,
        account: AccountToken,
        voucher_code: String,
    ) -> impl Future<Output = Result<VoucherSubmission, rest::Error>> {
        #[derive(serde::Serialize)]
        struct VoucherSubmission {
            voucher_code: String,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();
        let submission = VoucherSubmission { voucher_code };

        async move {
            let request = factory
                .post_json(&format!("{APP_URL_PREFIX}/submit-voucher"), &submission)?
                .account(account)?
                .expected_status(&[StatusCode::OK]);
            service.request(request).await?.deserialize().await
        }
    }

    #[cfg(target_os = "ios")]
    pub fn delete_account(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<(), rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .delete(&format!("{ACCOUNTS_URL_PREFIX}/accounts/me"))?
                .account(account.clone())?
                .header("Mullvad-Account-Number", &account)?
                .expected_status(&[StatusCode::NO_CONTENT]);

            let _ = service.request(request).await?;
            Ok(())
        }
    }

    #[cfg(target_os = "android")]
    pub fn init_play_purchase(
        &mut self,
        account: AccountToken,
    ) -> impl Future<Output = Result<PlayPurchasePaymentToken, rest::Error>> {
        #[derive(serde::Deserialize)]
        struct PlayPurchaseInitResponse {
            obfuscated_id: String,
        }

        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .post_json(&format!("{GOOGLE_PAYMENTS_URL_PREFIX}/init"), &())?
                .account(account)?
                .expected_status(&[StatusCode::OK]);
            let response = service.request(request).await?;

            let PlayPurchaseInitResponse { obfuscated_id } = response.deserialize().await?;

            Ok(obfuscated_id)
        }
    }

    #[cfg(target_os = "android")]
    pub fn verify_play_purchase(
        &mut self,
        account: AccountToken,
        play_purchase: PlayPurchase,
    ) -> impl Future<Output = Result<(), rest::Error>> {
        let service = self.handle.service.clone();
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .post_json(
                    &format!("{GOOGLE_PAYMENTS_URL_PREFIX}/acknowledge"),
                    &play_purchase,
                )?
                .account(account)?
                .expected_status(&[StatusCode::ACCEPTED]);
            service.request(request).await?;
            Ok(())
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

        async move {
            let request = factory
                .post(&format!("{APP_URL_PREFIX}/www-auth-token"))?
                .account(account)?
                .expected_status(&[StatusCode::OK]);
            let response = service.request(request).await?;
            let response: AuthTokenResponse = response.deserialize().await?;
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
        let factory = self.handle.factory.clone();

        async move {
            let request = factory
                .post_json(&format!("{APP_URL_PREFIX}/problem-report"), &report)?
                .expected_status(&[StatusCode::NO_CONTENT]);
            service.request(request).await?;
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
            let request = request?
                .expected_status(&[StatusCode::OK])
                .header("M-Platform-Version", &platform_version)?;
            let response = service.request(request).await?;
            response.deserialize().await
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
        let request = self
            .handle
            .factory
            .get(&format!("{APP_URL_PREFIX}/api-addrs"))?
            .expected_status(&[StatusCode::OK]);
        let response = self.handle.service.request(request).await?;
        response.deserialize().await
    }

    /// Check the availablility of `{APP_URL_PREFIX}/api-addrs`.
    pub async fn api_addrs_available(&self) -> Result<bool, rest::Error> {
        let request = self
            .handle
            .factory
            .head(&format!("{APP_URL_PREFIX}/api-addrs"))?
            .expected_status(&[StatusCode::OK]);

        let response = self.handle.service.request(request).await?;
        Ok(response.status().is_success())
    }
}
