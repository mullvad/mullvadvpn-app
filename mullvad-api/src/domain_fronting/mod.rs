//! Domain fronting for API connections.
//!
//! This module provides both client and server components for domain fronting,
//! allowing API connections to be tunneled through HTTP POST requests.
//!
//! # Client
//!
//! [`ProxyConnection`] implements [`AsyncRead`] + [`AsyncWrite`], tunneling data via HTTP POST requests.
//! The client establishes an HTTP/1.1 connection and uses POST requests with a session ID header
//! to maintain a bidirectional stream over HTTP.
//!
//! ## Usage
//!
//! ```no_run
//! use mullvad_api::domain_fronting::{DomainFronting, ProxyConfig};
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let df = DomainFronting::new(
//!     "cdn.example.com".to_string(),
//!     "api.example.com".to_string(),
//!     "X-Session-Id".to_string(),
//! );
//!
//! let proxy_config = df.proxy_config().await?;
//! let mut client = proxy_config.connect().await?;
//!
//! // Use like a regular AsyncRead + AsyncWrite stream
//! client.write_all(b"Hello").await?;
//! let mut buf = vec![0u8; 1024];
//! let n = client.read(&mut buf).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Server
//!
//! [`server::Sessions`] manages HTTP sessions, forwarding data to upstream servers.
//! Each unique session ID (sent via a configurable session header) gets its own
//! upstream TCP connection that persists across multiple HTTP requests.
//!
//! ## Usage
//!
//! ```no_run
//! use mullvad_api::domain_fronting::server::Sessions;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let upstream_addr = "127.0.0.1:8080".parse()?;
//! let sessions = Sessions::new(upstream_addr, "X-Session-Id".to_string());
//!
//! // Use with hyper to handle HTTP requests
//! // sessions.handle_request(req).await
//! # Ok(())
//! # }
//! ```
//!
//! # Testing
//!
//! Both client and server support generic [`AsyncRead`] + [`AsyncWrite`] streams for testing.
//! Use [`ProxyConnection::from_stream()`] and [`server::Sessions::with_connector()`] to inject
//! custom transports like [`tokio::io::duplex`] for unit tests.
//!
//! # Protocol
//!
//! - Each HTTP POST request contains data to send upstream
//! - Response body contains data received from upstream
//! - Empty POST requests are used for polling when no data needs to be sent
//! - Session cleanup happens when the client disconnects or the upstream closes

use std::{io, net::SocketAddr};

use crate::{DefaultDnsResolver, DnsResolver};

mod client;
pub mod server;

pub use client::{ProxyConfig, ProxyConnection};

/// Errors that can occur when establishing a domain fronting connection.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to establish TLS connection")]
    Tls(#[source] io::Error),
    #[error("HTTP handshake failed")]
    Handshake(#[from] hyper::Error),
    #[error("Connection failed")]
    Connection(#[source] io::Error),
    #[error("DNS resolution failed")]
    Dns(#[source] io::Error),
    #[error("Empty DNS response")]
    EmptyDnsResponse,
}

/// Configuration for creating a [`ProxyConfig`].
///
/// Contains the fronting domain, session header key and target host.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DomainFronting {
    /// Domain that will be used to connect to a CDN, used for SNI
    front: String,
    /// Host that will be reached via the CDN, i.e. this is the Host header value
    proxy_host: String,
    /// HTTP header key used to identify sessions
    session_header_key: String,
}

impl DomainFronting {
    pub fn new(front: String, proxy_host: String, session_header_key: String) -> Self {
        DomainFronting {
            front,
            proxy_host,
            session_header_key,
        }
    }

    /// Returns the fronting domain (used for SNI).
    pub fn front(&self) -> &str {
        &self.front
    }

    /// Returns the proxy host (used for Host header).
    pub fn proxy_host(&self) -> &str {
        &self.proxy_host
    }

    /// Returns the session header key.
    pub fn session_header_key(&self) -> &str {
        &self.session_header_key
    }

    pub async fn proxy_config(&self) -> Result<ProxyConfig, Error> {
        let dns_resolver = DefaultDnsResolver;

        let addrs = dns_resolver
            .resolve(self.front.clone())
            .await
            .map_err(Error::Dns)?;
        let addr = addrs.first().ok_or(Error::EmptyDnsResponse)?;

        Ok(ProxyConfig::new(
            SocketAddr::new(addr.ip(), 443),
            self.clone(),
        ))
    }
}
