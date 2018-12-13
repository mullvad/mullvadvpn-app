//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.


#[macro_use]
extern crate error_chain;

use chrono::{offset::Utc, DateTime};
use jsonrpc_client_core::{expand_params, jsonrpc_client};
use jsonrpc_client_http::{header::Host, HttpTransport, HttpTransportBuilder};
use lazy_static::lazy_static;
use mullvad_types::{account::AccountToken, relay_list::RelayList, version};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

pub mod event_loop;
pub mod rest;

mod cached_dns_resolver;
use crate::cached_dns_resolver::CachedDnsResolver;

mod https_client_with_sni;
use crate::https_client_with_sni::{HttpsClientWithSni, HttpsConnectorWithSni};

/// Number of threads in the thread pool doing DNS resolutions.
/// Since DNS is resolved via blocking syscall they must be run on separate threads.
const DNS_THREADS: usize = 2;

const API_HOST: &str = "api.mullvad.net";
const RPC_TIMEOUT: Duration = Duration::from_secs(5);
pub const API_IP_CACHE_FILENAME: &str = "api-ip-address.txt";
lazy_static! {
    static ref API_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 218, 73));
}


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcFactory {
    cached_dns_resolver: CachedDnsResolver,
    ca_path: PathBuf,
}

impl MullvadRpcFactory {
    /// Create a new `MullvadRpcFactory`.
    pub fn new<P: Into<PathBuf>>(ca_path: P) -> Self {
        MullvadRpcFactory {
            cached_dns_resolver: CachedDnsResolver::new(API_HOST.to_owned(), None, *API_IP),
            ca_path: ca_path.into(),
        }
    }

    /// Create a new `MullvadRpcFactory` using the specified cache directory.
    pub fn with_cache_dir<P: Into<PathBuf>>(cache_dir: &Path, ca_path: P) -> Self {
        let cache_file = cache_dir.join(API_IP_CACHE_FILENAME);
        let cached_dns_resolver =
            CachedDnsResolver::new(API_HOST.to_owned(), Some(cache_file), *API_IP);

        MullvadRpcFactory {
            cached_dns_resolver,
            ca_path: ca_path.into(),
        }
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
        F: FnOnce(
            HttpTransportBuilder<HttpsClientWithSni>,
        ) -> jsonrpc_client_http::Result<HttpTransport>,
    {
        let client = HttpsClientWithSni::new(API_HOST.to_owned(), self.ca_path.clone());
        let transport_builder = HttpTransportBuilder::with_client(client).timeout(RPC_TIMEOUT);

        let transport = create_transport(transport_builder)?;
        let api_uri = self.api_uri();
        log::debug!("Using API URI {}", api_uri);
        let mut handle = transport.handle(&api_uri)?;

        handle.set_header(Host::new(API_HOST, None));

        Ok(handle)
    }

    fn api_uri(&mut self) -> String {
        let ip = self.cached_dns_resolver.resolve().to_string();
        format!("https://{}/rpc/", ip)
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
