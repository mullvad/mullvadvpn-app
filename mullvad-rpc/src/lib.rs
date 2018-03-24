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
extern crate log;
extern crate native_tls;
extern crate serde_json;
extern crate tokio_core;

extern crate mullvad_types;

use chrono::DateTime;
use chrono::offset::Utc;
use jsonrpc_client_http::HttpTransport;
use jsonrpc_client_http::header::Host;
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;

use std::collections::HashMap;
use std::io;
use std::path::Path;

pub mod event_loop;
pub mod rest;

mod cached_dns_resolver;
use cached_dns_resolver::CachedDnsResolver;

static MASTER_API_HOST: &str = "api.mullvad.net";


/// A type that helps with the creation of RPC connections.
pub struct RpcConnectionManager {
    address_cache: Option<CachedDnsResolver>,
}

impl RpcConnectionManager {
    /// Create a new `RpcConnectionManager`.
    pub fn new() -> Self {
        RpcConnectionManager {
            address_cache: None,
        }
    }

    /// Create a new `RpcConnectionManager` using the specified resource and cache directories.
    pub fn with_resource_and_cache_dirs(resource_dir: &Path, cache_dir: &Path) -> io::Result<Self> {
        Ok(RpcConnectionManager {
            address_cache: Some(CachedDnsResolver::new(
                MASTER_API_HOST.to_owned(),
                cache_dir,
                resource_dir,
                "api_ip_address.txt",
            )?),
        })
    }

    /// Spawns a tokio core on a new thread and returns a `HttpHandle` running on that core.
    pub fn new_connection(&mut self) -> Result<HttpHandle, HttpError> {
        self.setup_connection(HttpTransport::new()?)
    }

    /// Create and returns a `HttpHandle` running on the given core handle.
    pub fn new_connection_on_event_loop(
        &mut self,
        handle: &Handle,
    ) -> Result<HttpHandle, HttpError> {
        self.setup_connection(HttpTransport::shared(handle)?)
    }

    fn setup_connection(&mut self, transport: HttpTransport) -> Result<HttpHandle, HttpError> {
        let uri = format!("https://{}/rpc/", self.api_address());
        let mut handle = transport.handle(&uri)?;

        handle.set_header(Host::new(MASTER_API_HOST, None));

        Ok(handle)
    }

    fn api_address(&mut self) -> String {
        if let Some(ref mut address_cache) = self.address_cache {
            address_cache.resolve().to_string()
        } else {
            MASTER_API_HOST.to_owned()
        }
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

impl ProblemReportProxy<HttpHandle> {
    pub fn connect(manager: &mut RpcConnectionManager) -> Result<Self, HttpError> {
        Ok(ProblemReportProxy::new(manager.new_connection()?))
    }
}

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});
