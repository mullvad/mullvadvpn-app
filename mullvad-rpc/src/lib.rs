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

use chrono::offset::Utc;
use chrono::DateTime;
use jsonrpc_client_http::header::Host;
use jsonrpc_client_http::HttpTransport;
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;
use mullvad_types::version;

use std::collections::HashMap;
use std::path::PathBuf;

pub mod event_loop;
pub mod rest;


static MASTER_API_HOST: &str = "api.mullvad.net";


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcFactory {
    resource_dir: Option<PathBuf>,
}

impl MullvadRpcFactory {
    /// Create a new `MullvadRpcFactory`.
    pub fn new() -> Self {
        MullvadRpcFactory { resource_dir: None }
    }

    /// Create a new `MullvadRpcFactory` using the specified resource directory.
    pub fn with_resource_dir(resource_dir: PathBuf) -> Self {
        MullvadRpcFactory {
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
        let uri = format!("https://{}/rpc/", MASTER_API_HOST);
        let mut handle = transport.handle(&uri)?;

        handle.set_header(Host::new(MASTER_API_HOST, None));

        Ok(handle)
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
    pub fn connect(manager: &MullvadRpcFactory) -> Result<Self, HttpError> {
        Ok(ProblemReportProxy::new(manager.new_connection()?))
    }
}

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});

jsonrpc_client!(pub struct AppVersionProxy {
    pub fn latest_app_version(&mut self) -> RpcRequest<version::LatestReleases>;
    pub fn is_app_version_supported(&mut self, version: &version::AppVersion) -> RpcRequest<bool>;
});
