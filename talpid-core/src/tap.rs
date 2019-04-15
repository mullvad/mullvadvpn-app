use crate::winnet;
use std::ffi::OsString;

/// Errors that this module may produce.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to get TAP interface alias.
    #[error(display = "Failed to get TAP interface alias")]
    FailedToGetTapInterfaceAlias(#[error(cause)] winnet::Error),
}

/// Use when creating talpid-types::net::openvpn::TunnelParameters.
pub fn resolve_tunnel_interface_alias() -> Result<Option<OsString>, Error> {
    #[cfg(windows)]
    {
        winnet::get_tap_interface_alias()
            .map_err(Error::FailedToGetTapInterfaceAlias)
            .map(Some)
    }
    #[cfg(not(windows))]
    {
        Ok(None)
    }
}
