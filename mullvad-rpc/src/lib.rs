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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

extern crate mullvad_types;

use chrono::DateTime;
use chrono::offset::Utc;
use hyper::header::Host;
use jsonrpc_client_http::HttpTransport;
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

pub mod event_loop;
pub mod rest;


static MASTER_API_HOST: &str = "api.mullvad.net";


#[derive(Deserialize, Serialize)]
struct AddressCacheData {
    ip: String,
    port: u16,
}

impl AddressCacheData {
    fn is_valid(&self) -> bool {
        self.ip.parse::<IpAddr>().is_ok()
    }
}

impl From<SocketAddr> for AddressCacheData {
    fn from(address: SocketAddr) -> Self {
        AddressCacheData {
            ip: address.ip().to_string(),
            port: address.port(),
        }
    }
}

impl Display for AddressCacheData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.ip, self.port)
    }
}

/// A type that helps with the creation of RPC connections.
pub struct RpcConnectionManager {
    resource_dir: Option<PathBuf>,
}

impl RpcConnectionManager {
    /// Create a new `RpcConnectionManager`.
    pub fn new() -> Self {
        RpcConnectionManager { resource_dir: None }
    }

    /// Create a new `RpcConnectionManager` using the specified resource directory.
    pub fn with_resource_dir(resource_dir: PathBuf) -> Self {
        RpcConnectionManager {
            resource_dir: Some(resource_dir),
        }
    }

    /// Spawns a tokio core on a new thread and returns a `HttpHandle` running on that core.
    pub fn new_connection(&self) -> Result<HttpHandle, HttpError> {
        self.setup_connection(HttpTransport::new()?)
    }

    /// Create and returns a `HttpHandle` running on the given core handle.
    pub fn new_connection_on_event_loop(&self, handle: &Handle) -> Result<HttpHandle, HttpError> {
        self.setup_connection(HttpTransport::shared(handle)?)
    }

    fn setup_connection(&self, transport: HttpTransport) -> Result<HttpHandle, HttpError> {
        let uri = format!("https://{}/rpc/", self.api_address());
        let mut handle = transport.handle(&uri)?;

        handle.set_header(Host::new(MASTER_API_HOST, None));

        Ok(handle)
    }

    fn api_address(&self) -> String {
        if let Some(cache_file) = self.cache_file_path() {
            Self::load_address_from_cache(&cache_file)
                .or_else(|_| {
                    Self::resolve_address().map(|address| {
                        let _ = Self::store_address_in_cache(address, &cache_file);
                        address.to_string()
                    })
                })
                .unwrap_or(MASTER_API_HOST.to_owned())
        } else {
            MASTER_API_HOST.to_owned()
        }
    }

    fn cache_file_path(&self) -> Option<PathBuf> {
        self.resource_dir
            .as_ref()
            .map(|dir| dir.join("api_address_cache.json"))
    }

    fn load_address_from_cache(cache_file_path: &Path) -> Result<String, io::Error> {
        let cache_file = File::open(cache_file_path)?;
        let address: AddressCacheData = serde_json::from_reader(cache_file)?;

        if address.is_valid() {
            Ok(address.to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "cached address is invalid",
            ))
        }
    }

    fn resolve_address() -> Result<SocketAddr, io::Error> {
        (MASTER_API_HOST, 0)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Mullvad RPC API host not found")
            })
    }

    fn store_address_in_cache(
        address: SocketAddr,
        cache_file_path: &Path,
    ) -> Result<(), io::Error> {
        let cache_file = File::create(cache_file_path)?;
        let cache_data = AddressCacheData::from(address);

        serde_json::to_writer(&cache_file, &cache_data)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
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
    pub fn connect(manager: &RpcConnectionManager) -> Result<Self, HttpError> {
        Ok(ProblemReportProxy::new(manager.new_connection()?))
    }
}

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});
