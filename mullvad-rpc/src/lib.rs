//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate serde_json;
extern crate tokio_core;

extern crate mullvad_types;

use chrono::offset::Utc;
use chrono::DateTime;
use jsonrpc_client_http::header::Host;
use jsonrpc_client_http::{HttpTransport, HttpTransportBuilder};
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;
use mullvad_types::version;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::time::Duration;

pub mod event_loop;
pub mod rest;

mod cached_dns_resolver;
use cached_dns_resolver::CachedDnsResolver;

mod https_client_with_sni;
use https_client_with_sni::HttpsClientWithSni;

const API_HOST: &str = "api.mullvad.net";
const RPC_TIMEOUT: Duration = Duration::from_secs(5);
pub const API_IP_CACHE_FILENAME: &str = "api-ip-address.txt";
lazy_static! {
    static ref API_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 219, 46));
}


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcFactory {
    address_cache: Option<CachedDnsResolver>,
}

impl MullvadRpcFactory {
    /// Create a new `MullvadRpcFactory`.
    pub fn new() -> Self {
        MullvadRpcFactory {
            address_cache: None,
        }
    }

    /// Create a new `MullvadRpcFactory` using the specified cache directory.
    pub fn with_cache_dir(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join(API_IP_CACHE_FILENAME);
        let cached_dns_resolver = CachedDnsResolver::new(API_HOST.to_owned(), cache_file, *API_IP);

        MullvadRpcFactory {
            address_cache: Some(cached_dns_resolver),
        }
    }

    /// Spawns a tokio core on a new thread and returns a `HttpHandle` running on that core.
    pub fn new_connection(&mut self) -> Result<HttpHandle, HttpError> {
        self.setup_connection(HttpTransportBuilder::standalone)
    }

    /// Create and returns a `HttpHandle` running on the given core handle.
    pub fn new_connection_on_event_loop(
        &mut self,
        handle: &Handle,
    ) -> Result<HttpHandle, HttpError> {
        self.setup_connection(move |transport| transport.shared(handle))
    }

    fn setup_connection<F>(&mut self, create_transport: F) -> Result<HttpHandle, HttpError>
    where
        F: FnOnce(HttpTransportBuilder<HttpsClientWithSni>)
            -> jsonrpc_client_http::Result<HttpTransport>,
    {
        let client = HttpsClientWithSni::new(API_HOST.to_owned());
        let transport_builder = HttpTransportBuilder::with_client(client).timeout(RPC_TIMEOUT);

        let transport = create_transport(transport_builder)?;
        let mut handle = transport.handle(&self.api_uri())?;

        handle.set_header(Host::new(API_HOST, None));

        Ok(handle)
    }

    fn api_uri(&mut self) -> String {
        let address = if let Some(ref mut address_cache) = self.address_cache {
            address_cache.resolve().to_string()
        } else {
            API_HOST.to_owned()
        };

        format!("https://{}/rpc/", address)
    }
}

jsonrpc_client!(pub struct AccountsProxy {
    pub fn get_expiry(&mut self, account_token: AccountToken) -> RpcRequest<DateTime<Utc>>;
});

jsonrpc_client!(pub struct ProblemReportProxy {
    pub fn problem_report(
        &mut self,
        email: &str,
        message: &str,
        log: &str,
        metadata: &HashMap<String, String>)
        -> RpcRequest<()>;
});

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});

jsonrpc_client!(pub struct AppVersionProxy {
    pub fn latest_app_version(&mut self) -> RpcRequest<version::LatestReleases>;
    pub fn is_app_version_supported(&mut self, version: &version::AppVersion) -> RpcRequest<bool>;
});
