#[macro_use]
extern crate error_chain;
extern crate mullvad_types;
extern crate serde;
extern crate talpid_ipc;
extern crate talpid_types;

use std::fs::{File, Metadata};
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
pub use platform_specific::rpc_file_path;

error_chain! {
    errors {
        EmptyRpcFile(file_path: String) {
            description("RPC connection file is empty")
            display("RPC connection file \"{}\" is empty", file_path)
        }

        InsecureRpcFile(file_path: String) {
            description(
                "RPC connection file is insecure because it might not have been written by an \
                administrator user"
            )
            display(
                "RPC connection file \"{}\" is insecure because it might not have been written by \
                an administrator user", file_path
            )
        }

        MissingRpcCredentials(file_path: String) {
            description("no credentials found in RPC connection file")
            display("no credentials found in RPC connection file {}", file_path)
        }

        ReadRpcFileError(file_path: String) {
            description("Failed to read RPC connection information")
            display("Failed to read RPC connection information from {}", file_path)
        }

        RpcCallError(method: String) {
            description("Failed to call RPC method")
            display("Failed to call RPC method \"{}\"", method)
        }

        StartRpcClient(address: String) {
            description("Failed to start RPC client")
            display("Failed to start RPC client to {}", address)
        }

        UnknownRpcFilePath {
            description("Failed to determine RPC connection information file path")
        }
    }
}

static NO_ARGS: [u8; 0] = [];

pub struct DaemonRpcClient {
    rpc_client: WsIpcClient,
}

impl DaemonRpcClient {
    pub fn new() -> Result<Self> {
        let (address, credentials) = Self::read_rpc_file()?;
        let rpc_client =
            WsIpcClient::connect(&address).chain_err(|| ErrorKind::StartRpcClient(address))?;

        Ok(DaemonRpcClient { rpc_client })
    }

    fn read_rpc_file() -> Result<(String, String)> {
        let file_path = rpc_file_path()?;
        let file_path_string = || file_path.display().to_string();
        let rpc_file =
            File::open(&file_path).chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))?;

        let file_metadata = rpc_file
            .metadata()
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))?;

        ensure_written_by_admin(&file_path, file_metadata)?;

        let reader = BufReader::new(rpc_file);
        let mut lines = reader.lines();

        let address = lines
            .next()
            .ok_or_else(|| ErrorKind::EmptyRpcFile(file_path_string()))?
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))?;
        let credentials = lines
            .next()
            .ok_or_else(|| ErrorKind::MissingRpcCredentials(file_path_string()))?
            .chain_err(|| ErrorKind::ReadRpcFileError(file_path_string()))?;

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

    pub fn rpc_file_path() -> Result<PathBuf> {
        Ok(Path::new("/tmp/.mullvad_rpc_address").to_path_buf())
    }

    pub fn ensure_written_by_admin<P: AsRef<Path>>(file_path: P, metadata: Metadata) -> Result<()> {
        let is_owned_by_root = metadata.uid() == 0;
        let is_read_only_by_non_owner = (metadata.mode() & 0o022) == 0;

        ensure!(
            is_owned_by_root && is_read_only_by_non_owner,
            ErrorKind::InsecureRpcFile(file_path.as_ref().display().to_string())
        );

        Ok(())
    }
}

#[cfg(windows)]
mod platform_specific {
    use super::*;

    pub fn rpc_file_path() -> Result<PathBuf> {
        let windows_directory =
            ::std::env::var_os("WINDIR").ok_or_else(|| ErrorKind::UnknownRpcFilePath)?;

        Ok(PathBuf::from(windows_directory)
            .join("Temp")
            .join(".mullvad_rpc_address"))
    }

    pub fn ensure_written_by_admin<P: AsRef<Path>>(
        _file_path: P,
        _metadata: Metadata,
    ) -> Result<()> {
        // TODO: Check permissions correctly
        Ok(())
    }
}
