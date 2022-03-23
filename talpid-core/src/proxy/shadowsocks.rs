pub use std::io::Result;

use crate::logging;
use regex::Regex;

use std::{
    borrow::Cow,
    env,
    ffi::OsString,
    fmt,
    fs::File,
    io::{BufRead, Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use super::{ProxyMonitor, ProxyMonitorCloseHandle, ProxyResourceData, WaitResult};
use talpid_types::net::openvpn::ShadowsocksProxySettings;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;

struct ShadowsocksCommand {
    shadowsocks_bin: OsString,
    local: Option<SocketAddr>,
    peer: Option<SocketAddr>,
    peer_password: Option<String>,
    // This should map to the shadowsocks-rust `CipherType` type.
    cipher: Option<String>,
}

impl ShadowsocksCommand {
    pub fn new(shadowsocks_bin: OsString) -> Self {
        ShadowsocksCommand {
            shadowsocks_bin,
            local: None,
            peer: None,
            peer_password: None,
            cipher: None,
        }
    }

    pub fn local(&mut self, local: SocketAddr) -> &mut Self {
        self.local = Some(local);
        self
    }

    pub fn peer(&mut self, peer: SocketAddr) -> &mut Self {
        self.peer = Some(peer);
        self
    }

    pub fn peer_password(&mut self, password: String) -> &mut Self {
        self.peer_password = Some(password);
        self
    }

    pub fn cipher(&mut self, cipher: String) -> &mut Self {
        self.cipher = Some(cipher);
        self
    }

    pub fn build(&self) -> duct::Expression {
        log::debug!("Building expression: {}", &self);
        duct::cmd(&self.shadowsocks_bin, self.get_arguments()).unchecked()
    }

    fn get_arguments(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![];

        // Always activate TCP no-delay.
        args.push("--no-delay".to_owned());

        if let Some(ref local) = self.local {
            args.push("--local-addr".to_owned());
            args.push(format!("{}:{}", local.ip(), local.port()));
        }

        if let Some(ref peer) = self.peer {
            args.push("--server-addr".to_owned());
            args.push(format!("{}:{}", peer.ip(), peer.port()));
        }

        if let Some(ref peer_password) = self.peer_password {
            args.push("--password".to_owned());
            args.push(peer_password.to_owned());
        }

        if let Some(ref cipher) = self.cipher {
            args.push("--encrypt-method".to_owned());
            args.push(cipher.to_string());
        }

        args
    }
}

impl fmt::Display for ShadowsocksCommand {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&shell_escape::escape(
            self.shadowsocks_bin.to_string_lossy(),
        ))?;
        for arg in &self.get_arguments() {
            fmt.write_str(" ")?;
            fmt.write_str(&shell_escape::escape(Cow::from(arg)))?;
        }
        Ok(())
    }
}

pub struct ShadowsocksProxyMonitor {
    subproc: Arc<ProcessHandle>,
    closed: Arc<AtomicBool>,
    port: u16,
}

const SHADOWSOCKS_LOG_FILENAME: &str = "shadowsocks.log";
#[cfg(unix)]
const SHADOWSOCKS_BIN_FILENAME: &str = "sslocal";
#[cfg(windows)]
const SHADOWSOCKS_BIN_FILENAME: &str = "sslocal.exe";

struct ProcessHandle {
    subproc: duct::Handle,
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        let _ = self.subproc.kill();
    }
}

impl std::ops::Deref for ProcessHandle {
    type Target = duct::Handle;

    fn deref(&self) -> &Self::Target {
        &self.subproc
    }
}

impl ShadowsocksProxyMonitor {
    pub fn start(
        settings: &ShadowsocksProxySettings,
        resource_data: &ProxyResourceData,
    ) -> Result<Self> {
        let binary = resource_data
            .resource_dir
            .join(SHADOWSOCKS_BIN_FILENAME)
            .into_os_string();

        let mut cmd = ShadowsocksCommand::new(binary)
            .local(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
            .peer(settings.peer)
            .peer_password(settings.password.clone())
            .cipher(settings.cipher.clone())
            .build();

        let log_dir: PathBuf = if let Some(ref log_dir) = resource_data.log_dir {
            log_dir.clone()
        } else {
            env::temp_dir()
        };

        let logfile = log_dir.join(SHADOWSOCKS_LOG_FILENAME);

        logging::rotate_log(&logfile)
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to rotate log file"))?;

        cmd = cmd.stdin_null().stderr_to_stdout().stdout_path(&logfile);

        let subproc = cmd.start()?;

        #[cfg(target_os = "linux")]
        {
            // Run this process outside the tunnel
            use crate::split_tunnel::PidManager;

            let excluded_pids = PidManager::new().map_err(|error| {
                Error::new(
                    ErrorKind::Other,
                    error.display_chain_with_msg("Failed to initialize PidManager"),
                )
            })?;
            for pid in subproc.pids() {
                excluded_pids.add(pid as i32).map_err(|error| {
                    Error::new(
                        ErrorKind::Other,
                        error.display_chain_with_msg("Failed to exclude Shadowsocks process"),
                    )
                })?;
            }
        }

        match Self::get_bound_port(File::open(&logfile)?, &subproc) {
            Ok(port) => Ok(Self {
                subproc: Arc::new(ProcessHandle { subproc }),
                closed: Arc::new(AtomicBool::new(false)),
                port,
            }),
            Err(err) => {
                let _ = subproc.kill();
                Err(err)
            }
        }
    }

    fn get_bound_port(logfile: File, subproc: &duct::Handle) -> Result<u16> {
        let mut buffered_reader = std::io::BufReader::new(logfile);

        for _tries in 0..5 {
            loop {
                // `read_line` appends to the buffer so keep a small scope for the `line` variable.
                let mut line = String::new();
                match buffered_reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }
                        // `read_line` includes the line break in the returned line.
                        if let Ok(port) = Self::parse_port(line.trim_end()) {
                            return Ok(port);
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            if subproc.try_wait().unwrap().is_some() {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }

        Err(Error::new(
            ErrorKind::Other,
            "Could not determine which port Shadowsocks has bound to",
        ))
    }

    fn parse_port(logline: &str) -> Result<u16> {
        // TODO: Compile once and reuse.
        let re = Regex::new(r"(?:TCP listening on \d+\.\d+\.\d+\.\d+:)(\d+$)").unwrap();

        if let Some(captures) = re.captures(logline) {
            return Ok(captures[1].parse().map_err(|_| {
                Error::new(ErrorKind::Other, "Failed to parse port number string")
            })?);
        }

        Err(Error::new(ErrorKind::Other, "No port number present"))
    }
}

impl ProxyMonitor for ShadowsocksProxyMonitor {
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle> {
        Box::new(ShadowsocksProxyMonitorCloseHandle {
            subproc: self.subproc.clone(),
            closed: self.closed.clone(),
        })
    }

    fn wait(self: Box<Self>) -> Result<WaitResult> {
        self.subproc.wait().map(|output| {
            if self.closed.load(Ordering::SeqCst) {
                Ok(WaitResult::ProperShutdown)
            } else {
                Ok(WaitResult::UnexpectedExit(
                    if let Some(exit_code) = output.status.code() {
                        format!("Exit code: {}", exit_code)
                    } else {
                        "Exit code is indeterminable".to_string()
                    },
                ))
            }
        })?
    }

    fn port(&self) -> u16 {
        self.port
    }
}

pub struct ShadowsocksProxyMonitorCloseHandle {
    subproc: Arc<ProcessHandle>,
    closed: Arc<AtomicBool>,
}

impl ProxyMonitorCloseHandle for ShadowsocksProxyMonitorCloseHandle {
    fn close(self: Box<Self>) -> Result<()> {
        if !self.closed.swap(true, Ordering::SeqCst) {
            self.subproc.kill()
        } else {
            Ok(())
        }
    }
}
