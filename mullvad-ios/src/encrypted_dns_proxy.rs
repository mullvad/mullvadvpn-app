use crate::ProxyHandle;

use libc::c_char;
use mullvad_encrypted_dns_proxy::{
    state::{EncryptedDnsProxyState as State, FetchConfigError},
    Forwarder,
};
use std::{
    io, mem,
    net::{Ipv4Addr, SocketAddr},
    ptr,
};
use tokio::{net::TcpListener, task::JoinHandle};

use std::ffi::CStr;

/// A thin wrapper around [`mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState`] that
/// can start a local forwarder (see [`Self::start`]).
pub struct EncryptedDnsProxyState {
    state: State,
    domain: String,
}

#[derive(Debug)]
pub enum Error {
    /// Failed to initialize tokio runtime.
    TokioRuntime,
    /// Failed to bind a local listening socket, the one that will be forwarded through the proxy.
    BindLocalSocket(io::Error),
    /// Failed to get local listening address of the local listening socket.
    GetBindAddr(io::Error),
    /// Failed to initialize forwarder.
    Forwarder(io::Error),
    /// Failed to fetch a proxy configuration over DNS.
    FetchConfig(FetchConfigError),
    /// Failed to initialize with a valid configuration.
    NoConfigs,
}

impl From<Error> for i32 {
    fn from(err: Error) -> Self {
        match err {
            Error::TokioRuntime => -1,
            Error::BindLocalSocket(_) => -2,
            Error::GetBindAddr(_) => -3,
            Error::Forwarder(_) => -4,
            Error::FetchConfig(_) => -5,
            Error::NoConfigs => -6,
        }
    }
}

impl EncryptedDnsProxyState {
    async fn start(&mut self) -> Result<ProxyHandle, Error> {
        self.state
            .fetch_configs(&self.domain)
            .await
            .map_err(Error::FetchConfig)?;
        let proxy_configuration = self.state.next_configuration().ok_or(Error::NoConfigs)?;

        let local_socket = Self::bind_local_addr()
            .await
            .map_err(Error::BindLocalSocket)?;
        let bind_addr = local_socket.local_addr().map_err(Error::GetBindAddr)?;
        let forwarder = Forwarder::connect(&proxy_configuration)
            .await
            .map_err(Error::Forwarder)?;
        let join_handle = Box::new(tokio::spawn(async move {
            if let Ok((client_conn, _)) = local_socket.accept().await {
                let _ = forwarder.forward(client_conn).await;
            }
        }));

        Ok(ProxyHandle {
            context: Box::into_raw(join_handle).cast(),
            port: bind_addr.port(),
        })
    }

    async fn bind_local_addr() -> io::Result<TcpListener> {
        let bind_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
        TcpListener::bind(bind_addr).await
    }
}

/// Initializes a valid pointer to an instance of `EncryptedDnsProxyState`.
///
/// # Safety
///
/// * [domain_name] must not be non-null.
///
/// * [domain_name] pointer must be [valid](core::ptr#safety)
///
/// * The caller must ensure that the pointer to the [domain_name] string contains a nul terminator
///   at the end of the string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encrypted_dns_proxy_init(
    domain_name: *const c_char,
) -> *mut EncryptedDnsProxyState {
    let domain = {
        // SAFETY: domain_name points to a valid region of memory and contains a nul terminator.
        let c_str = unsafe { CStr::from_ptr(domain_name) };
        String::from_utf8_lossy(c_str.to_bytes())
    };

    let state = Box::new(EncryptedDnsProxyState {
        state: State::default(),
        domain: domain.into_owned(),
    });
    Box::into_raw(state)
}

/// This must be called only once to deallocate `EncryptedDnsProxyState`.
///
/// # Safety
/// `ptr` must be a valid, exclusive pointer to `EncryptedDnsProxyState`, initialized
/// by `encrypted_dns_proxy_init`. This function is not thread safe, and should only be called
/// once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encrypted_dns_proxy_free(ptr: *mut EncryptedDnsProxyState) {
    // SAFETY: See notes above
    let _ = unsafe { Box::from_raw(ptr) };
}

/// # Safety
/// encrypted_dns_proxy must be a valid, exclusive pointer to `EncryptedDnsProxyState`, initialized
/// by `encrypted_dns_proxy_init`. This function is not thread safe.
/// `proxy_handle` must be pointing to a valid memory region for the size of a `ProxyHandle`. This
/// function is not thread safe, but it can be called repeatedly. Each successful invocation should
/// clean up the resulting proxy via `[encrypted_dns_proxy_stop]`.
///
/// `proxy_handle` will only contain valid values if the return value is zero. It is still valid to
/// deallocate the memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encrypted_dns_proxy_start(
    encrypted_dns_proxy: *mut EncryptedDnsProxyState,
    proxy_handle: *mut ProxyHandle,
) -> i32 {
    let handle = match crate::mullvad_ios_runtime() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("Cannot instantiate a tokio runtime: {}", err);
            return Error::TokioRuntime.into();
        }
    };

    // SAFETY: See notes above
    let mut encrypted_dns_proxy = unsafe { Box::from_raw(encrypted_dns_proxy) };
    let proxy_result = handle.block_on(encrypted_dns_proxy.start());
    mem::forget(encrypted_dns_proxy);

    match proxy_result {
        // SAFETY: `proxy_handle` is guaranteed to be a valid pointer
        Ok(handle) => unsafe { ptr::write(proxy_handle, handle) },
        Err(err) => {
            let empty_handle = ProxyHandle {
                context: ptr::null_mut(),
                port: 0,
            };
            // SAFETY: `proxy_handle` is guaranteed to be a valid pointer
            unsafe { ptr::write(proxy_handle, empty_handle) }
            log::error!("Failed to create a proxy connection: {err:?}");
            return err.into();
        }
    }

    0
}

/// # Safety
/// `proxy_config` must be a valid pointer to a `ProxyHandle` as initialized by
/// [`encrypted_dns_proxy_start`]. It should only ever be called once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encrypted_dns_proxy_stop(proxy_config: *mut ProxyHandle) -> i32 {
    // SAFETY: See notes above
    let ptr = unsafe { (*proxy_config).context };
    if !ptr.is_null() {
        // SAFETY: `ptr` is guaranteed to be non-null and valid
        let handle: Box<JoinHandle<()>> = unsafe { Box::from_raw(ptr.cast()) };
        handle.abort();
    }
    0i32
}
