use chrono::{offset::Utc, DateTime};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};

/// Identifier used to authenticate or identify a Mullvad account.
pub type AccountToken = String;

/// Account expiration info returned by the API via `/v1/me`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct AccountData {
    #[cfg_attr(target_os = "android", jnix(map = "|expiry| expiry.to_string()"))]
    pub expiry: DateTime<Utc>,
}

/// Data structure that's returned from successful invocation of the mullvad API's
/// `/v1/submit-voucher` RPC.
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct VoucherSubmission {
    /// Amount of time added to the account
    #[cfg_attr(target_os = "android", jnix(map = "|time_added| time_added as i64"))]
    pub time_added: u64,
    /// Updated expiry time
    #[cfg_attr(target_os = "android", jnix(map = "|expiry| expiry.to_string()"))]
    pub new_expiry: DateTime<Utc>,
}

/// Mapping of mullvad-api errors to enum variants. Used by frontends to explain why a voucher
/// was rejected by the `/v1/submit-voucher` RPC.
#[derive(err_derive::Error, Debug)]
pub enum VoucherError {
    /// Error code `tonic::Code::NotFound`
    #[error(display = "Bad voucher code")]
    BadVoucher,
    /// Error code `tonic::Code::ResourceExhausted`
    #[error(display = "Voucher already used")]
    VoucherAlreadyUsed,
    /// Error code `tonic::Code::Internal`
    #[error(display = "Server internal error")]
    InternalError,
    #[error(display = "Unknown error, {}", _0)]
    UnknownError(i64),
}

impl VoucherError {
    /// Create error from RPC error code.
    pub fn from_rpc_error_code(err_code: i64) -> VoucherError {
        match err_code {
            x if x == tonic::Code::NotFound as i64 => VoucherError::BadVoucher,
            x if x == tonic::Code::ResourceExhausted as i64 => VoucherError::VoucherAlreadyUsed,
            x if x == tonic::Code::Internal as i64 => VoucherError::InternalError,
            err => VoucherError::UnknownError(err),
        }
    }
}
