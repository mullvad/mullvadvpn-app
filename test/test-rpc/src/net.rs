use futures::channel::oneshot;
use hyper::{Client, Uri};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::SocketAddr;

use tokio_rustls::rustls::ClientConfig;

use crate::{AmIMullvad, Error};

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

static CLIENT_CONFIG: Lazy<ClientConfig> = Lazy::new(|| {
    ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(read_cert_store())
        .with_no_client_auth()
});

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SockHandleId(pub usize);

pub struct SockHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    bind_addr: SocketAddr,
}

impl SockHandle {
    pub(crate) async fn start_tcp_forward(
        client: crate::service::ServiceClient,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<Self, Error> {
        let (stop_tx, stop_rx) = oneshot::channel();

        let (id, bind_addr) = client
            .start_tcp_forward(tarpc::context::current(), bind_addr, via_addr)
            .await??;

        tokio::spawn(async move {
            let _ = stop_rx.await;

            log::trace!("Stopping TCP forward");

            if let Err(error) = client.stop_tcp_forward(tarpc::context::current(), id).await {
                log::error!("Failed to stop TCP forward: {error}");
            }
        });

        Ok(SockHandle {
            stop_tx: Some(stop_tx),
            bind_addr,
        })
    }

    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

impl Drop for SockHandle {
    fn drop(&mut self) {
        self.stop()
    }
}

pub async fn geoip_lookup(mullvad_host: String) -> Result<AmIMullvad, Error> {
    let uri = Uri::try_from(format!("https://ipv4.am.i.{mullvad_host}/json"))
        .map_err(|_| Error::InvalidUrl)?;
    http_get(uri).await
}

pub async fn http_get<T: DeserializeOwned>(url: Uri) -> Result<T, Error> {
    log::debug!("GET {url}");

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(CLIENT_CONFIG.clone())
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, hyper::Body> = Client::builder().build(https);
    let body = client
        .get(url)
        .await
        .map_err(|error| Error::HttpRequest(error.to_string()))?
        .into_body();

    // TODO: limit length
    let bytes = hyper::body::to_bytes(body).await.map_err(|error| {
        log::error!("Failed to convert body to bytes buffer: {}", error);
        Error::DeserializeBody
    })?;

    serde_json::from_slice(&bytes).map_err(|error| {
        log::error!("Failed to deserialize response: {}", error);
        Error::DeserializeBody
    })
}

fn read_cert_store() -> tokio_rustls::rustls::RootCertStore {
    let mut cert_store = tokio_rustls::rustls::RootCertStore::empty();

    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(LE_ROOT_CERT))
        .expect("Failed to parse pem file");
    let (num_certs_added, num_failures) = cert_store.add_parsable_certificates(&certs);
    if num_failures > 0 || num_certs_added != 1 {
        panic!("Failed to add root cert");
    }

    cert_store
}

#[cfg(unix)]
pub mod unix {
    use std::{io, os::fd::AsRawFd};
    // TODO: These functions are copied/derived from `talpid_wireguard::unix`, since we don't want
    // to depend on the entire `talpid_wireguard` crate. Perhaps they should be moved to a new
    // crate, e.g. `talpid_unix`?

    #[cfg(target_os = "macos")]
    const SIOCGIFMTU: u64 = 0xc0206933;
    #[cfg(target_os = "linux")]
    const SIOCGIFMTU: u64 = libc::SIOCGIFMTU;

    // #[cfg(target_os = "macos")]
    pub fn get_mtu(interface_name: &str) -> Result<u16, io::Error> {
        let sock = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;

        let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
        if interface_name.len() >= ifr.ifr_name.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Interface name too long",
            ));
        }

        // SAFETY: interface_name is shorter than ifr.ifr_name
        unsafe {
            std::ptr::copy_nonoverlapping(
                interface_name.as_ptr() as *const libc::c_char,
                &mut ifr.ifr_name as *mut _,
                interface_name.len(),
            )
        };

        // TODO: define SIOCGIFMTU for macos
        // SAFETY: SIOCGIFMTU expects an ifreq, and the socket is valid
        if unsafe { libc::ioctl(sock.as_raw_fd(), SIOCGIFMTU, &mut ifr) } < 0 {
            let e = io::Error::last_os_error();
            log::error!("SIOCGIFMTU failed: {}", e);
            return Err(e);
        }

        // SAFETY: ifru_mtu is set since SIOGCIFMTU succeeded
        Ok(unsafe { ifr.ifr_ifru.ifru_mtu }
            .try_into()
            .expect("MTU should fit in u16"))
    }

    #[cfg(target_os = "macos")]
    const SIOCSIFMTU: u64 = 0x80206934;
    #[cfg(target_os = "linux")]
    const SIOCSIFMTU: u64 = libc::SIOCSIFMTU;

    pub fn set_mtu(interface_name: &str, mtu: u16) -> Result<(), io::Error> {
        debug_assert_ne!(
            interface_name, "eth0",
            "Should be name of mullvad tunnel interface, e.g. 'wg0-mullvad'"
        );

        let sock = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;

        let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
        if interface_name.len() >= ifr.ifr_name.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Interface name too long",
            ));
        }

        // SAFETY: interface_name is shorter than ifr.ifr_name
        unsafe {
            std::ptr::copy_nonoverlapping(
                interface_name.as_ptr() as *const libc::c_char,
                &mut ifr.ifr_name as *mut _,
                interface_name.len(),
            )
        };
        ifr.ifr_ifru.ifru_mtu = mtu as i32;

        // SAFETY: SIOCGIFMTU expects an ifreq, and the socket is valid
        if unsafe { libc::ioctl(sock.as_raw_fd(), SIOCSIFMTU, &ifr) } < 0 {
            let e = io::Error::last_os_error();
            log::error!("SIOCSIFMTU failed: {}", e);
            return Err(e);
        }
        Ok(())
    }
}
