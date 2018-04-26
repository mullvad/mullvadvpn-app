#[macro_use]
extern crate error_chain;
extern crate mullvad_paths;
extern crate mullvad_types;
extern crate serde;
extern crate talpid_ipc;
extern crate talpid_types;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::location::GeoIpLocation;
use mullvad_types::relay_constraints::{RelaySettings, RelaySettingsUpdate};
use mullvad_types::relay_list::RelayList;
use mullvad_types::states::DaemonState;
use mullvad_types::version::AppVersionInfo;
use serde::{Deserialize, Serialize};
use talpid_ipc::WsIpcClient;
use talpid_types::net::TunnelOptions;

use platform_specific::ensure_written_by_admin;

error_chain! {
    errors {
        AuthenticationError {
            description("Failed to authenticate the connection with the daemon")
        }

        EmptyRpcFile(path: PathBuf) {
            description("RPC connection file is empty")
            display("RPC connection file \"{}\" is empty", path.display())
        }

        InsecureRpcFile(path: PathBuf) {
            description(
                "RPC connection file is insecure because it might not have been written by an \
                administrator user"
            )
            display(
                "RPC connection file \"{}\" is insecure because it might not have been written by \
                an administrator user", path.display()
            )
        }

        MissingRpcCredentials(path: PathBuf) {
            description("no credentials found in RPC connection file")
            display("no credentials found in RPC connection file {}", path.display())
        }

        ReadRpcFileError(path: PathBuf) {
            description("Failed to read RPC connection information")
            display("Failed to read RPC connection information from {}", path.display())
        }

        RpcCallError(method: String) {
            description("Failed to call RPC method")
            display("Failed to call RPC method \"{}\"", method)
        }

        StartRpcClient(address: String) {
            description("Failed to start RPC client")
            display("Failed to start RPC client to {}", address)
        }
    }
    links {
        UnknownRpcAddressPath(mullvad_paths::Error, mullvad_paths::ErrorKind);
    }
}

static NO_ARGS: [u8; 0] = [];

pub struct DaemonRpcClient {
    rpc_client: WsIpcClient,
}

impl DaemonRpcClient {
    pub fn new() -> Result<Self> {
        let (address, credentials) = Self::read_rpc_file(true)?;

        Self::with_address_and_credentials(address, credentials)
    }

    pub fn without_rpc_file_security_check() -> Result<Self> {
        let (address, credentials) = Self::read_rpc_file(false)?;

        Self::with_address_and_credentials(address, credentials)
    }

    fn with_address_and_credentials(address: String, credentials: String) -> Result<Self> {
        let rpc_client =
            WsIpcClient::connect(&address).chain_err(|| ErrorKind::StartRpcClient(address))?;
        let mut instance = DaemonRpcClient { rpc_client };

        instance
            .auth(&credentials)
            .chain_err(|| ErrorKind::AuthenticationError)?;

        Ok(instance)
    }

    fn read_rpc_file(verify_security: bool) -> Result<(String, String)> {
        let file_path = mullvad_paths::get_rpc_address_path()?;
        let rpc_file =
            File::open(&file_path).chain_err(|| ErrorKind::ReadRpcFileError(file_path.clone()))?;

        if verify_security {
            ensure_written_by_admin(&file_path)?;
        }

        let reader = BufReader::new(rpc_file);
        let mut lines = reader.lines();

        let address = lines
            .next()
            .ok_or_else(|| ErrorKind::EmptyRpcFile(file_path.clone()))?
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path.clone()))?;
        let credentials = lines
            .next()
            .ok_or_else(|| ErrorKind::MissingRpcCredentials(file_path.clone()))?
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path.clone()))?;

        Ok((address, credentials))
    }

    pub fn auth(&mut self, credentials: &str) -> Result<()> {
        self.call("auth", &[credentials])
    }

    pub fn connect(&mut self) -> Result<()> {
        self.call("connect", &NO_ARGS)
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.call("disconnect", &NO_ARGS)
    }

    pub fn get_account(&mut self) -> Result<Option<AccountToken>> {
        self.call("get_account", &NO_ARGS)
    }

    pub fn get_account_data(&mut self, account: AccountToken) -> Result<AccountData> {
        self.call("get_account_data", &[account])
    }

    pub fn get_allow_lan(&mut self) -> Result<bool> {
        self.call("get_allow_lan", &NO_ARGS)
    }

    pub fn get_current_location(&mut self) -> Result<GeoIpLocation> {
        self.call("get_current_location", &NO_ARGS)
    }

    pub fn get_current_version(&mut self) -> Result<String> {
        self.call("get_current_version", &NO_ARGS)
    }

    pub fn get_relay_locations(&mut self) -> Result<RelayList> {
        self.call("get_relay_locations", &NO_ARGS)
    }

    pub fn get_relay_settings(&mut self) -> Result<RelaySettings> {
        self.call("get_relay_settings", &NO_ARGS)
    }

    pub fn get_state(&mut self) -> Result<DaemonState> {
        self.call("get_state", &NO_ARGS)
    }

    pub fn get_tunnel_options(&mut self) -> Result<TunnelOptions> {
        self.call("get_tunnel_options", &NO_ARGS)
    }

    pub fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        self.call("get_version_info", &NO_ARGS)
    }

    pub fn set_account(&mut self, account: Option<AccountToken>) -> Result<()> {
        self.call("set_account", &[account])
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<()> {
        self.call("set_allow_lan", &[allow_lan])
    }

    pub fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Result<()> {
        self.call("set_openvpn_mssfix", &[mssfix])
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.call("shutdown", &NO_ARGS)
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<()> {
        self.call("update_relay_settings", &[update])
    }

    pub fn call<A, O>(&mut self, method: &str, args: &A) -> Result<O>
    where
        A: Serialize,
        O: for<'de> Deserialize<'de>,
    {
        self.rpc_client
            .call(method, args)
            .chain_err(|| ErrorKind::RpcCallError(method.to_owned()))
    }
}

#[cfg(unix)]
mod platform_specific {
    use std::os::unix::fs::MetadataExt;

    use super::*;

    pub fn ensure_written_by_admin<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        let metadata = path
            .metadata()
            .chain_err(|| ErrorKind::ReadRpcFileError(path.to_owned()))?;

        let is_owned_by_root = metadata.uid() == 0;
        let is_read_only_by_non_owner = (metadata.mode() & 0o022) == 0;

        ensure!(
            is_owned_by_root && is_read_only_by_non_owner,
            ErrorKind::InsecureRpcFile(path.to_owned())
        );

        Ok(())
    }
}

#[cfg(windows)]
mod platform_specific {
    use super::*;

    pub fn ensure_written_by_admin<P: AsRef<Path>>(_file_path: P) -> Result<()> {
        // TODO: Check permissions correctly
        Ok(())
    }
}
