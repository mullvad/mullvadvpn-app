//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate chrono;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate serde_json;

extern crate mullvad_types;

use chrono::DateTime;
use chrono::offset::Utc;
use jsonrpc_client_http::HttpTransport;

pub use jsonrpc_client_core::{Error, ErrorKind};
pub use jsonrpc_client_http::{Error as HttpError, HttpHandle};

use mullvad_types::account::AccountToken;
use mullvad_types::relay_list::RelayList;

use std::collections::HashMap;


static MASTER_API_URI: &str = "https://api.mullvad.net/rpc/";


pub fn connect() -> Result<HttpHandle, HttpError> {
    HttpTransport::new()?.handle(MASTER_API_URI)
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
        let transport = HttpTransport::new()?.handle(MASTER_API_URI)?;
        Ok(ProblemReportProxy::new(transport))
    }
}

jsonrpc_client!(pub struct RelayListProxy {
    pub fn relay_list(&mut self) -> RpcRequest<RelayList>;
});
