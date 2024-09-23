use crate::ProxyHandle;

use mullvad_encrypted_dns_proxy::{config::ProxyConfig, config_resolver, Forwarder};
use std::{
    collections::HashSet,
    io, mem,
    net::{Ipv4Addr, SocketAddr},
    ptr,
};
use tokio::{net::TcpListener, task::JoinHandle};

pub struct EncryptedDnsProxyState {
    /// Note that we rely on the randomness of the ordering of the items in the hashset to pick a
    /// random configurations every time.
    configurations: HashSet<ProxyConfig>,
    tried_configurations: HashSet<ProxyConfig>,
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
    FetchConfig(config_resolver::Error),
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
        self.fetch_configs().await?;
        let proxy_configuration = self.next_configuration().ok_or(Error::NoConfigs)?;

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

    /// Select a config.
    /// Always select an obfuscated configuration, if there are any left untried. If no obfuscated
    /// configurations exist, try plain configurations. The order is randomized due to the hash set
    /// storing the configurations in a random order.
    fn next_configuration(&mut self) -> Option<ProxyConfig> {
        if self.should_reset() {
            self.reset();
        }

        // First, try getting an obfuscated configuration, if there exist any.
        let config = if let Some(obfuscated_config) = self
            .configurations
            .difference(&self.tried_configurations)
            .find(|config| config.obfuscation.is_some())
            .cloned()
        {
            obfuscated_config
        } else {
            // If no obfuscated configurations exist, try the next untried configuration.
            self.configurations
                .difference(&self.tried_configurations)
                .next()
                .cloned()?
        };

        self.tried_configurations.insert(config.clone());
        Some(config)
    }

    /// Fetch a config, but error out only when no existing configuration was there.
    async fn fetch_configs(&mut self) -> Result<(), Error> {
        match mullvad_encrypted_dns_proxy::config_resolver::resolve_default_config().await {
            Ok(new_configs) => {
                self.configurations = HashSet::from_iter(new_configs.into_iter());
            }
            Err(err) => {
                log::error!("Failed to fetch a new proxy configuration: {err:?}");
                if self.is_empty() {
                    return Err(Error::FetchConfig(err));
                }
            }
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.configurations.is_empty()
    }

    /// Checks if the `tried_configurations` set should be reset.
    /// It should only be reset if the difference between `configurations` and
    /// `tried_configurations` is an empty set - in this case all available configurations have
    /// been tried.
    fn should_reset(&self) -> bool {
        self.configurations
            .difference(&self.tried_configurations)
            .count()
            == 0
    }

    /// Clears the `tried_configurations` set.
    fn reset(&mut self) {
        self.tried_configurations.clear();
    }
}

/// Initializes a valid pointer to an instance of `EncryptedDnsProxyState`.
#[no_mangle]
pub unsafe extern "C" fn encrypted_dns_proxy_init() -> *mut EncryptedDnsProxyState {
    let state = Box::new(EncryptedDnsProxyState {
        configurations: Default::default(),
        tried_configurations: Default::default(),
    });
    Box::into_raw(state)
}

/// This must be called only once to deallocate `EncryptedDnsProxyState`.
///
/// # Safety
/// `ptr` must be a valid, exclusive pointer to `EncryptedDnsProxyState`, initialized
/// by `encrypted_dns_proxy_init`. This function is not thread safe, and should only be called
/// once.
#[no_mangle]
pub unsafe extern "C" fn encrypted_dns_proxy_free(ptr: *mut EncryptedDnsProxyState) {
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
#[no_mangle]
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

    let mut encrypted_dns_proxy = unsafe { Box::from_raw(encrypted_dns_proxy) };
    let proxy_result = handle.block_on(encrypted_dns_proxy.start());
    mem::forget(encrypted_dns_proxy);

    match proxy_result {
        Ok(handle) => unsafe { ptr::write(proxy_handle, handle) },
        Err(err) => {
            let empty_handle = ProxyHandle {
                context: ptr::null_mut(),
                port: 0,
            };
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
#[no_mangle]
pub unsafe extern "C" fn encrypted_dns_proxy_stop(proxy_config: *mut ProxyHandle) -> i32 {
    let ptr = unsafe { (*proxy_config).context };
    if !ptr.is_null() {
        let handle: Box<JoinHandle<()>> = unsafe { Box::from_raw(ptr.cast()) };
        handle.abort();
    }
    0i32
}
