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
use std::path::Path;

pub mod event_loop;
pub mod rest;

mod api_address;
use api_address::*;


/// Create and returns a `HttpHandle` running on the given core handle.
pub fn shared(handle: &Handle, resource_dir: Option<&Path>) -> Result<HttpHandle, HttpError> {
    create_http_handle(resource_dir, HttpTransport::shared(handle)?)
}

/// Spawns a tokio core on a new thread and returns a `HttpHandle` running on that core.
pub fn standalone(resource_dir: Option<&Path>) -> Result<HttpHandle, HttpError> {
    create_http_handle(resource_dir, HttpTransport::new()?)
}

fn create_http_handle(
    resource_dir: Option<&Path>,
    transport: HttpTransport,
) -> Result<HttpHandle, HttpError> {
    let uri = format!("https://{}/rpc", api_address(resource_dir));
    let mut handle = transport.handle(&uri)?;
    handle.set_header(Host::new(MASTER_API_HOST, None));
    Ok(handle)
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
    pub fn connect() -> Result<Self, HttpError> {
        let transport = standalone(None)?;
        Ok(ProblemReportProxy::new(transport))
    }
}

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});
