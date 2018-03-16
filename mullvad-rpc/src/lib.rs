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
use jsonrpc_client_http::HttpTransport;
use tokio_core::reactor::Handle;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;
use mullvad_types::version;

use std::collections::HashMap;

pub mod event_loop;
pub mod rest;


static MASTER_API_URI: &str = "https://api.mullvad.net/rpc/";


/// A type that helps with the creation of RPC connections.
pub struct MullvadRpcFactory;

impl MullvadRpcFactory {
    /// Create a new `MullvadRpcFactory`.
    pub fn new() -> Self {
        MullvadRpcFactory
    }

    /// Spawns a tokio core on a new thread and returns a `HttpHandle` running on that core.
    pub fn new_connection(&self) -> Result<HttpHandle, HttpError> {
        HttpTransport::new()?.handle(MASTER_API_URI)
    }

    /// Create and returns a `HttpHandle` running on the given core handle.
    pub fn new_connection_on_event_loop(&self, handle: &Handle) -> Result<HttpHandle, HttpError> {
        HttpTransport::shared(handle)?.handle(MASTER_API_URI)
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
