//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![recursion_limit = "128"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_json;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_ws_server;
extern crate url;
extern crate ws;

use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_ws_server::{MetaExtractor, NoopExtractor, Server, ServerBuilder};

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};


mod client;
pub use client::*;

/// An Id created by the Ipc server that the client can use to connect to it
pub type IpcServerId = String;

error_chain!{
    errors {
        IpcServerError {
            description("Error in IPC server")
        }
    }
}


pub struct IpcServer {
    address: String,
    server: Server,
}

impl IpcServer {
    pub fn start<M: Metadata>(handler: MetaIoHandler<M>) -> Result<Self> {
        Self::start_with_metadata(handler, NoopExtractor)
    }

    pub fn start_with_metadata<M, E>(handler: MetaIoHandler<M>, meta_extractor: E) -> Result<Self>
    where
        M: Metadata,
        E: MetaExtractor<M>,
    {
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        ServerBuilder::new(handler)
            .session_meta_extractor(meta_extractor)
            .start(&listen_addr)
            .map(|server| IpcServer {
                address: format!("ws://{}", server.addr()),
                server: server,
            }).chain_err(|| ErrorKind::IpcServerError)
    }

    /// Returns the localhost address this `IpcServer` is listening on.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Creates a handle bound to this `IpcServer` that can be used to shut it down.
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle(self.server.close_handle())
    }

    /// Consumes the server and waits for it to finish. Get an `CloseHandle` before calling this
    /// if you want to be able to shut the server down.
    pub fn wait(self) -> Result<()> {
        self.server.wait().chain_err(|| ErrorKind::IpcServerError)
    }
}

// FIXME: This custom impl is because `Server` does not implement `Debug` yet:
// https://github.com/paritytech/jsonrpc/pull/195
impl fmt::Debug for IpcServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IpcServer")
            .field("address", &self.address)
            .finish()
    }
}


#[derive(Clone)]
pub struct CloseHandle(jsonrpc_ws_server::CloseHandle);

impl CloseHandle {
    pub fn close(self) {
        self.0.close();
    }
}
