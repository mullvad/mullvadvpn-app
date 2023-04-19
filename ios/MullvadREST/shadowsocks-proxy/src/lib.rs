use shadowsocks_service::{
    config::{
        Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig,
    },
    local::Server,
    shadowsocks::{config::ServerConfig, crypto::CipherKind},
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener},
    str::FromStr,
    sync::Once,
};
use tokio::sync::oneshot;

use shadowsocks_service;

const LOCAL_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
const INIT_LOGGING: Once = Once::new();

pub fn run_http_proxy(
    bridge_addr: SocketAddr,
    password: &str,
    cipher: &str,
) -> io::Result<(u16, ShadowsocksHandle)> {
    let runtime = ShadowsocksRuntime::new(bridge_addr, password, cipher)?;
    let port = runtime.port();
    let handle = runtime.run()?;

    Ok((port, handle))
}

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut libc::c_void,
    pub port: u16,
}

struct ShadowsocksRuntime {
    runtime: tokio::runtime::Runtime,
    config: Config,
    local_port: u16,
}

pub struct ShadowsocksHandle {
    tx: oneshot::Sender<()>,
}

impl ShadowsocksHandle {
    pub fn stop(self) {
        let _ = self.tx.send(());
    }
}

impl ShadowsocksRuntime {
    pub fn new(bridge_addr: SocketAddr, password: &str, cipher: &str) -> io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let (config, local_port) = Self::create_config(bridge_addr, password, cipher)?;
        Ok(Self {
            runtime,
            config,
            local_port,
        })
    }

    pub fn port(&self) -> u16 {
        self.local_port
    }

    pub fn run(self) -> io::Result<ShadowsocksHandle> {
        let (tx, rx) = oneshot::channel();
        let (startup_tx, startup_rx) = oneshot::channel();
        std::thread::spawn(move || {
            self.run_service_inner(rx, startup_tx);
        });

        match startup_rx.blocking_recv() {
            Ok(Ok(_)) => Ok(ShadowsocksHandle { tx }),
            Ok(Err(err)) => {
                let _ = tx.send(());
                Err(err)
            }
            Err(_) => {
                let _ = tx.send(());
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Tokio runtime crashed",
                ))
            }
        }
    }

    fn run_service_inner(
        self,
        rx: oneshot::Receiver<()>,
        startup_done_tx: oneshot::Sender<io::Result<()>>,
    ) {
        let Self {
            config, runtime, ..
        } = self;

        std::thread::spawn(move || {
            runtime.spawn(async move {
                match Server::create(config).await {
                    Ok(server) => {
                        let _ = startup_done_tx.send(Ok(()));
                        let _ = server.wait_until_exit().await;
                    }
                    Err(err) => {
                        let _ = startup_done_tx.send(Err(err));
                    }
                }
            });
            let _ = runtime.block_on(rx);
        });
    }

    pub fn create_config(
        bridge_addr: SocketAddr,
        password: &str,
        cipher: &str,
    ) -> io::Result<(Config, u16)> {
        let mut cfg = Config::new(ConfigType::Local);
        let free_port = get_free_port()?;
        let bind_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), free_port);

        cfg.local = vec![LocalInstanceConfig::with_local_config(
            LocalConfig::new_with_addr(bind_addr.into(), ProtocolType::Http),
        )];

        let cipher = match CipherKind::from_str(cipher) {
            Ok(cipher) => cipher,
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid cipher specified: {}", err),
                ));
            }
        };
        let server_config = ServerInstanceConfig::with_server_config(ServerConfig::new(
            bridge_addr,
            password,
            cipher,
        ));

        cfg.server = vec![server_config];

        Ok((cfg, free_port))
    }
}

#[no_mangle]
pub extern "C" fn start_shadowsocks_proxy(
    addr: *const u8,
    addr_len: usize,
    port: u16,
    password: *const u8,
    password_len: usize,
    cipher: *const u8,
    cipher_len: usize,
    proxy_config: *mut ProxyHandle,
) -> i32 {
    INIT_LOGGING.call_once(|| {
        oslog::OsLogger::new("net.mullvad.MullvadVPN.HTTPProxy")
            .level_filter(log::LevelFilter::Debug)
            .init()
            .unwrap();
    });

    let bridge_ip = if let Some(addr) = unsafe { parse_ip_addr(addr, addr_len) } {
        addr
    } else {
        return -1;
    };

    let bridge_addr = SocketAddr::new(bridge_ip, port);

    let password = if let Some(password) = unsafe { parse_str(password, password_len) } {
        password
    } else {
        return -1;
    };

    let cipher = if let Some(cipher) = unsafe { parse_str(cipher, cipher_len) } {
        cipher
    } else {
        return -1;
    };

    let (port, handle) = match run_http_proxy(bridge_addr, &password, &cipher) {
        Ok((port, handle)) => (port, handle),
        Err(err) => {
            log::error!("Failed to run HTTP proxy {}", err);
            return err.raw_os_error().unwrap_or(-1);
        }
    };
    let handle = Box::new(handle);

    unsafe {
        std::ptr::write(
            proxy_config,
            ProxyHandle {
                port,
                context: Box::into_raw(handle) as *mut _,
            },
        )
    }

    0
}

#[no_mangle]
pub extern "C" fn stop_shadowsocks_proxy(proxy_config: *mut ProxyHandle) -> i32 {
    let context_ptr = unsafe { (*proxy_config).context };
    if context_ptr.is_null() {
        return -1;
    }

    let proxy_handle: Box<ShadowsocksHandle> = unsafe { Box::from_raw(context_ptr as *mut _) };
    proxy_handle.stop();
    0
}

fn get_free_port() -> io::Result<u16> {
    let port = TcpListener::bind(LOCAL_ADDR)?.local_addr()?.port();
    Ok(port)
}

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` must be a pointer to at least `addr_len` bytes.
unsafe fn parse_ip_addr(addr: *const u8, addr_len: usize) -> Option<IpAddr> {
    match addr_len {
        4 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).into())
        }
        16 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            let mut addr_arr = [0u8; 16];
            addr_arr.as_mut_slice().copy_from_slice(&bytes);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}

/// Allocates a new string with the contents of `data` if it contains only valid UTF-8 bytes.
///
/// SAFETY: `data` must be a valid pointer to `len` amount of bytes
unsafe fn parse_str(data: *const u8, len: usize) -> Option<String> {
    // SAFETY: data pointer must be valid for the size of len
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    String::from_utf8(bytes.to_vec()).ok()
}
