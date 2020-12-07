#![deny(rust_2018_idioms)]

use chrono::{offset::Utc, DateTime};
use hyper::Method;
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    version::AppVersion,
};
use std::{
    collections::BTreeMap,
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    sync::Arc,
};
use talpid_types::{net::wireguard, ErrorExt};


pub mod rest;

mod https_client_with_sni;
use crate::https_client_with_sni::HttpsConnectorWithSni;

mod address_cache;
mod relay_list;
pub use address_cache::{AddressCache, CurrentAddressChangeListener};
pub use hyper::StatusCode;
pub use relay_list::RelayListProxy;

/// Error code returned by the Mullvad API if the voucher has alreaby been used.
pub const VOUCHER_USED: &str = "VOUCHER_USED";

/// Error code returned by the Mullvad API if the voucher code is invalid.
pub const INVALID_VOUCHER: &str = "INVALID_VOUCHER";

const API_HOST: &str = "api.mullvad.net";
pub const API_IP_CACHE_FILENAME: &str = "api-ip-address.txt";
const API_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 218, 78));
const API_ADDRESS: (IpAddr, u16) = (crate::API_IP, 443);


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcRuntime {
    https_connector: HttpsConnectorWithSni,
    handle: tokio::runtime::Handle,
    pub address_cache: AddressCache,
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to construct a rest client")]
    RestError(#[error(source)] rest::Error),

    #[error(display = "Failed to load address cache")]
    AddressCacheError(#[error(source)] address_cache::Error),
}

impl MullvadRpcRuntime {
    /// Create a new `MullvadRpcRuntime`.
    pub fn new(handle: tokio::runtime::Handle) -> Result<Self, Error> {
        Ok(MullvadRpcRuntime {
            https_connector: HttpsConnectorWithSni::new(),
            handle,
            address_cache: AddressCache::new(
                vec![API_ADDRESS.into()],
                None,
                Arc::new(Box::new(|_| Ok(()))),
            )?,
        })
    }

    /// Create a new `MullvadRpcRuntime` using the specified directories.
    /// Try to use the cache directory first, and fall back on the resource directory
    /// if it fails.
    pub async fn with_cache(
        handle: tokio::runtime::Handle,
        resource_dir: Option<&Path>,
        cache_dir: &Path,
        write_changes: bool,
        address_change_listener: impl Fn(SocketAddr) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        let cache_file = cache_dir.join(API_IP_CACHE_FILENAME);
        let write_file = if write_changes {
            Some(cache_file.clone().into_boxed_path())
        } else {
            None
        };

        let address_change_listener =
            Arc::<Box<CurrentAddressChangeListener>>::new(Box::new(address_change_listener));

        let address_cache = match AddressCache::from_file(
            &cache_file,
            write_file.clone(),
            address_change_listener.clone(),
        )
        .await
        {
            Ok(cache) => cache,
            Err(error) => {
                let cache_exists = cache_file.exists();
                if cache_exists {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to load cached API addresses. Falling back on bundled list"
                        )
                    );
                }

                // Initialize the cache directory cache using the resource directory
                match resource_dir {
                    Some(resource_dir) => {
                        let read_file = resource_dir.join(API_IP_CACHE_FILENAME);
                        let empty_listener =
                            Arc::<Box<CurrentAddressChangeListener>>::new(Box::new(|_| Ok(())));
                        let mut cache =
                            AddressCache::from_file(&read_file, write_file, empty_listener).await?;
                        cache.randomize().await?;
                        cache.set_change_listener(address_change_listener);
                        cache
                    }
                    None => return Err(Error::AddressCacheError(error)),
                }
            }
        };

        let https_connector = HttpsConnectorWithSni::new();

        Ok(MullvadRpcRuntime {
            https_connector,
            handle,
            address_cache,
        })
    }

    /// Creates a new request service and returns a handle to it.
    fn new_request_service(&mut self, sni_hostname: Option<String>) -> rest::RequestServiceHandle {
        let mut https_connector = self.https_connector.clone();
        https_connector.set_sni_hostname(sni_hostname);

        let service = rest::RequestService::new(
            https_connector,
            self.handle.clone(),
            self.address_cache.clone(),
        );
        let handle = service.handle();
        self.handle.spawn(service.into_future());
        handle
    }

    /// Returns a request factory initialized to create requests for the master API
    pub fn mullvad_rest_handle(&mut self) -> rest::MullvadRestHandle {
        let service = self.new_request_service(Some(API_HOST.to_owned()));
        let factory = rest::RequestFactory::new(
            API_HOST.to_owned(),
            Box::new(self.address_cache.clone()),
            Some("app".to_owned()),
        );

        rest::MullvadRestHandle::new(service, factory, self.address_cache.clone())
    }

    /// Returns a new request service handle
    pub fn rest_handle(&mut self) -> rest::RequestServiceHandle {
        self.new_request_service(None)
    }

    pub fn handle(&mut self) -> &mut tokio::runtime::Handle {
        &mut self.handle
    }
}

pub struct AccountsProxy {
    handle: rest::MullvadRestHandle,
}

#[derive(serde::Deserialize)]
struct AccountResponse {
    token: AccountToken,
    expires: DateTime<Utc>,
}

impl AccountsProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn get_expiry(
        &self,
        account: AccountToken,
    ) -> impl Future<Output = Result<DateTime<Utc>, rest::Error>> {
        let service = self.handle.service.clone();

        let response = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/me",
            Method::GET,
            Some(account),
            StatusCode::OK,
        );
        async move {
            let account: AccountResponse = rest::deserialize_body(response.await?).await?;
            Ok(account.expires)
        }
    }

    pub fn create_account(&mut self) -> impl Future<Output = Result<AccountToken, rest::Error>> {
        let service = self.handle.service.clone();
        let response = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/accounts",
            Method::POST,
            None,
            StatusCode::CREATED,
        );

        async move {
            let account: AccountResponse = rest::deserialize_body(response.await?).await?;
            Ok(account.token)
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
        let submission = VoucherSubmission { voucher_code };

        let response = rest::post_request_with_json(
            &self.handle.factory,
            service,
            "/v1/submit-voucher",
            &submission,
            Some(account_token),
            StatusCode::OK,
        );

        async move { rest::deserialize_body(response.await?).await }
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
        let response = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/www-auth-token",
            Method::POST,
            Some(account),
            StatusCode::OK,
        );

        async move {
            let response: AuthTokenResponse = rest::deserialize_body(response.await?).await?;
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

        let request = rest::post_request_with_json(
            &self.handle.factory,
            service,
            "/v1/problem-report",
            &report,
            None,
            StatusCode::NO_CONTENT,
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
}

impl AppVersionProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn version_check(
        &self,
        version: AppVersion,
        platform: &str,
    ) -> impl Future<Output = Result<AppVersionResponse, rest::Error>> {
        let service = self.handle.service.clone();

        let request = rest::send_request(
            &self.handle.factory,
            service,
            &format!("/v1/releases/{}/{}", platform, version),
            Method::GET,
            None,
            StatusCode::OK,
        );

        async move { rest::deserialize_body(request.await?).await }
    }
}


/// Error code for when an account has too many keys. Returned when trying to push a new key.
pub const KEY_LIMIT_REACHED: &str = "KEY_LIMIT_REACHED";
#[derive(Clone)]
pub struct WireguardKeyProxy {
    handle: rest::MullvadRestHandle,
}


impl WireguardKeyProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn push_wg_key(
        &mut self,
        account_token: AccountToken,
        public_key: wireguard::PublicKey,
        timeout: Option<std::time::Duration>,
    ) -> impl Future<Output = Result<mullvad_types::wireguard::AssociatedAddresses, rest::Error>> + 'static
    {
        #[derive(serde::Serialize)]
        struct PublishRequest {
            pubkey: wireguard::PublicKey,
        }

        let service = self.handle.service.clone();
        let body = PublishRequest { pubkey: public_key };

        let request = self.handle.factory.post_json(&"/v1/wireguard-keys", &body);
        async move {
            let mut request = request?;
            if let Some(timeout) = timeout {
                request.set_timeout(timeout);
            }
            request.set_auth(Some(account_token))?;
            let response = service.request(request).await?;
            rest::deserialize_body(rest::parse_rest_response(response, StatusCode::CREATED).await?)
                .await
        }
    }

    pub async fn replace_wg_key(
        &mut self,
        account_token: AccountToken,
        old: wireguard::PublicKey,
        new: wireguard::PublicKey,
    ) -> Result<mullvad_types::wireguard::AssociatedAddresses, rest::Error> {
        #[derive(serde::Serialize)]
        struct ReplacementRequest {
            old: wireguard::PublicKey,
            new: wireguard::PublicKey,
        }

        let service = self.handle.service.clone();
        let body = ReplacementRequest { old, new };

        let response = rest::post_request_with_json(
            &self.handle.factory,
            service,
            &"/v1/replace-wireguard-key",
            &body,
            Some(account_token),
            StatusCode::CREATED,
        )
        .await?;

        rest::deserialize_body(response).await
    }

    pub async fn get_wireguard_key(
        &mut self,
        account_token: AccountToken,
        key: &wireguard::PublicKey,
    ) -> Result<mullvad_types::wireguard::AssociatedAddresses, rest::Error> {
        let service = self.handle.service.clone();

        let response = rest::send_request(
            &self.handle.factory,
            service,
            &format!(
                "/v1/wireguard-keys/{}",
                urlencoding::encode(&key.to_base64())
            ),
            Method::GET,
            Some(account_token),
            StatusCode::OK,
        )
        .await?;

        rest::deserialize_body(response).await
    }

    pub async fn remove_wireguard_key(
        &mut self,
        account_token: AccountToken,
        key: &wireguard::PublicKey,
    ) -> Result<(), rest::Error> {
        let service = self.handle.service.clone();

        let _ = rest::send_request(
            &self.handle.factory,
            service,
            &format!(
                "/v1/wireguard-keys/{}",
                urlencoding::encode(&key.to_base64())
            ),
            Method::DELETE,
            Some(account_token),
            StatusCode::NO_CONTENT,
        )
        .await?;
        Ok(())
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
            "/v1/api-addrs",
            Method::GET,
            None,
            StatusCode::OK,
        )
        .await?;

        rest::deserialize_body(response).await
    }
}
