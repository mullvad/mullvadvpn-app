#![deny(rust_2018_idioms)]

use chrono::{offset::Utc, DateTime};
use hyper::Method;
use mullvad_types::{
    account::{AccountToken, VoucherSubmission},
    version,
};
use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr},
    path::Path,
};
use talpid_types::net::wireguard;


pub mod event_loop;
pub mod rest;

mod cached_dns_resolver;
use crate::cached_dns_resolver::CachedDnsResolver;

mod https_client_with_sni;
use crate::https_client_with_sni::HttpsConnectorWithSni;

mod relay_list;
pub use hyper::StatusCode;
pub use relay_list::RelayListProxy;


const API_HOST: &str = "api.mullvad.net";
pub const API_IP_CACHE_FILENAME: &str = "api-ip-address.txt";
const API_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 218, 78));


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcRuntime {
    cached_dns_resolver: CachedDnsResolver,
    https_connector: HttpsConnectorWithSni,
    runtime: tokio::runtime::Runtime,
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to construct a connector")]
    ConnectorError(#[error(source)] https_client_with_sni::Error),
    #[error(display = "Failed to construct a rest client")]
    RestError(#[error(source)] rest::Error),
    #[error(display = "Failed to spawn a tokio runtime")]
    TokioRuntimeError(#[error(source)] tokio::io::Error),
}

impl MullvadRpcRuntime {
    /// Create a new `MullvadRpcRuntime`.
    pub fn new(ca_path: &Path) -> Result<Self, Error> {
        let https_connector =
            HttpsConnectorWithSni::new(&ca_path).map_err(Error::ConnectorError)?;
        Ok(MullvadRpcRuntime {
            cached_dns_resolver: CachedDnsResolver::new(API_HOST.to_owned(), None, API_IP),
            runtime: event_loop::create_runtime()?,
            https_connector,
        })
    }

    /// Create a new `MullvadRpcRuntime` using the specified cache directory.
    pub fn with_cache_dir(cache_dir: &Path, ca_path: &Path) -> Result<Self, Error> {
        let cache_file = cache_dir.join(API_IP_CACHE_FILENAME);
        let cached_dns_resolver =
            CachedDnsResolver::new(API_HOST.to_owned(), Some(cache_file), API_IP);

        let https_connector =
            HttpsConnectorWithSni::new(&ca_path).map_err(Error::ConnectorError)?;

        Ok(MullvadRpcRuntime {
            cached_dns_resolver,
            runtime: event_loop::create_runtime()?,
            https_connector,
        })
    }

    /// Creates a new request service and returns a handle to it.
    fn new_request_service(&mut self, sni_hostname: Option<String>) -> rest::RequestServiceHandle {
        let mut https_connector = self.https_connector.clone();
        https_connector.set_sni_hostname(sni_hostname);

        let service = rest::RequestService::new(https_connector, self.runtime.handle().clone());
        let handle = service.handle();
        self.runtime.spawn(service.into_future());
        handle
    }

    /// Returns a request factory initialized to create requests for the master API
    pub fn mullvad_rest_handle(&mut self) -> rest::MullvadRestHandle {
        let service = self.new_request_service(Some(API_HOST.to_owned()));
        let ip = self.cached_dns_resolver.resolve();
        let factory =
            rest::RequestFactory::new(API_HOST.to_owned(), Some(ip), Some("app".to_owned()));

        rest::MullvadRestHandle { service, factory }
    }

    /// Returns a new request service handle
    pub fn rest_handle(&mut self) -> rest::RequestServiceHandle {
        self.new_request_service(None)
    }
}

impl Drop for MullvadRpcRuntime {
    fn drop(&mut self) {
        if let Ok(runtime) = event_loop::create_runtime() {
            let old_runtime = std::mem::replace(&mut self.runtime, runtime);
            old_runtime.shutdown_timeout(std::time::Duration::from_secs(1));
        }
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
pub const VOUCHER_USED: &str = "VOUCHER_USED";
pub const INVALID_VOUCHER: &str = "INVALID_VOUCHER";
pub const MISSING_ARGUMENT: &str = "MISSING_ARGUMENT";

impl AccountsProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn get_expiry(
        &self,
        account: AccountToken,
    ) -> impl futures01::future::Future<Item = DateTime<Utc>, Error = rest::Error> {
        let service = self.handle.service.clone();

        let response = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/me",
            Method::GET,
            Some(account),
            StatusCode::OK,
        );
        self.handle.service.compat_spawn(async move {
            let account: AccountResponse = rest::deserialize_body(response.await?).await?;
            Ok(account.expires)
        })
    }

    pub fn create_account(
        &mut self,
    ) -> impl futures01::future::Future<Item = AccountToken, Error = rest::Error> {
        let service = self.handle.service.clone();
        let response = rest::send_request(
            &self.handle.factory,
            service,
            "/v1/accounts",
            Method::POST,
            None,
            StatusCode::CREATED,
        );

        self.handle.service.compat_spawn(async move {
            let account: AccountResponse = rest::deserialize_body(response.await?).await?;
            Ok(account.token)
        })
    }

    pub fn submit_voucher(
        &mut self,
        account_token: AccountToken,
        voucher_code: String,
    ) -> impl futures01::future::Future<Item = VoucherSubmission, Error = rest::Error> {
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

        self.handle
            .service
            .compat_spawn(async move { rest::deserialize_body(response.await?).await })
    }

    pub fn get_www_auth_token(
        &self,
        account: AccountToken,
    ) -> impl futures01::future::Future<Item = String, Error = rest::Error> {
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

        let future = async move {
            let response: AuthTokenResponse = rest::deserialize_body(response.await?).await?;
            Ok(response.auth_token)
        };

        self.handle.service.compat_spawn(future)
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
    ) -> impl futures01::future::Future<Item = (), Error = rest::Error> {
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

        self.handle.service.compat_spawn(async move {
            request.await?;
            Ok(())
        })
    }
}

pub struct AppVersionProxy {
    handle: rest::MullvadRestHandle,
}

impl AppVersionProxy {
    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn version_check(
        &self,
        version: version::AppVersion,
        platform: &str,
    ) -> impl futures01::future::Future<Item = mullvad_types::version::AppVersionInfo, Error = rest::Error>
    {
        let service = self.handle.service.clone();

        let request = rest::send_request(
            &self.handle.factory,
            service,
            &format!("/v1/releases/{}/{}", platform, version),
            Method::GET,
            None,
            StatusCode::OK,
        );
        self.handle
            .service
            .compat_spawn(async move { rest::deserialize_body(request.await?).await })
    }
}


/// Error code for when an account has too many keys. Returned when trying to push a new key.
pub const KEY_LIMIT_REACHED: &str = "KEY_LIMIT_REACHED";
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
    ) -> impl futures01::future::Future<
        Item = mullvad_types::wireguard::AssociatedAddresses,
        Error = rest::Error,
    > {
        #[derive(serde::Serialize)]
        struct PublishRequest {
            pubkey: wireguard::PublicKey,
        }

        let service = self.handle.service.clone();
        let body = PublishRequest { pubkey: public_key };

        let request = rest::post_request_with_json(
            &self.handle.factory,
            service,
            &"/v1/wireguard-keys",
            &body,
            Some(account_token),
            StatusCode::CREATED,
        );
        self.handle
            .service
            .compat_spawn(async move { rest::deserialize_body(request.await?).await })
    }

    pub fn replace_wg_key(
        &mut self,
        account_token: AccountToken,
        old: wireguard::PublicKey,
        new: wireguard::PublicKey,
    ) -> impl futures01::future::Future<
        Item = mullvad_types::wireguard::AssociatedAddresses,
        Error = rest::Error,
    > {
        #[derive(serde::Serialize)]
        struct ReplacementRequest {
            old: wireguard::PublicKey,
            new: wireguard::PublicKey,
        }

        let service = self.handle.service.clone();
        let body = ReplacementRequest { old, new };

        let request = rest::post_request_with_json(
            &self.handle.factory,
            service,
            &"/v1/replace-wireguard-key",
            &body,
            Some(account_token),
            StatusCode::CREATED,
        );

        self.handle
            .service
            .compat_spawn(async move { rest::deserialize_body(request.await?).await })
    }

    pub fn get_wireguard_key(
        &mut self,
        account_token: AccountToken,
        key: &wireguard::PublicKey,
    ) -> impl futures01::future::Future<
        Item = mullvad_types::wireguard::AssociatedAddresses,
        Error = rest::Error,
    > {
        let service = self.handle.service.clone();

        let request = rest::send_request(
            &self.handle.factory,
            service,
            &format!(
                "/v1/wireguard-keys/{}",
                urlencoding::encode(&key.to_base64())
            ),
            Method::GET,
            Some(account_token),
            StatusCode::OK,
        );
        self.handle
            .service
            .compat_spawn(async move { rest::deserialize_body(request.await?).await })
    }

    pub fn remove_wireguard_key(
        &mut self,
        account_token: AccountToken,
        key: &wireguard::PublicKey,
    ) -> impl futures01::future::Future<Item = (), Error = rest::Error> {
        let service = self.handle.service.clone();

        let request = rest::send_request(
            &self.handle.factory,
            service,
            &format!(
                "/v1/wireguard-keys/{}",
                urlencoding::encode(&key.to_base64())
            ),
            Method::DELETE,
            Some(account_token),
            StatusCode::NO_CONTENT,
        );

        self.handle.service.compat_spawn(async move {
            let _ = request.await?;
            Ok(())
        })
    }
}
