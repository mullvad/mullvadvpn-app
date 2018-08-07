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
extern crate jsonrpc_ipc_server;
extern crate jsonrpc_pubsub;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_ipc;
extern crate tokio_core;
extern crate url;
extern crate ws;

use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_ipc_server::{MetaExtractor, NoopExtractor, Server, ServerBuilder};

use std::fmt;


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
    path: String,
    server: Server,
}

impl IpcServer {
    pub fn start<M: Metadata + Default>(handler: MetaIoHandler<M>, path: String) -> Result<Self> {
        Self::start_with_metadata(handler, NoopExtractor, path)
    }

    pub fn start_with_metadata<M, E>(
        handler: MetaIoHandler<M>,
        meta_extractor: E,
        path: String,
    ) -> Result<Self>
    where
        M: Metadata + Default,
        E: MetaExtractor<M>,
    {
        ServerBuilder::new(handler)
            .session_meta_extractor(meta_extractor)
            .start(&path)
            .map(|server| IpcServer {
                path: path.to_owned(),
                server: server,
            }).chain_err(|| ErrorKind::IpcServerError)
    }

    /// Returns the uds/named pipe path this `IpcServer` is listening on.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Creates a handle bound to this `IpcServer` that can be used to shut it down.
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle(self.server.close_handle())
    }

    /// Consumes the server and waits for it to finish. Get a `CloseHandle` before calling this
    /// if you want to be able to shut the server down.
    pub fn wait(self) {
        self.server.wait()
    }
}

// FIXME: This custom impl is because `Server` does not implement `Debug` yet:
// https://github.com/paritytech/jsonrpc/pull/195
impl fmt::Debug for IpcServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IpcServer")
            .field("path", &self.path)
            .finish()
    }
}


#[derive(Clone)]
pub struct CloseHandle(jsonrpc_ipc_server::CloseHandle);

impl CloseHandle {
    pub fn close(self) {
        self.0.close();
    }
}
