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
pub struct VoucherSubmission {
    /// Amount of time added to the account
    pub time_added: u64,
    /// Updated expiry time
    pub new_expiry: DateTime<Utc>,
}

/// Mapping of mullvad-api errors to enum variants. Used by frontends to explain why a voucher
/// was rejected by the `/v1/submit-voucher` RPC.
#[derive(err_derive::Error, Debug)]
pub enum VoucherError {
    /// Error code -400
    #[error(display = "Bad voucher code")]
    BadVoucher,
    /// Error code -401
    #[error(display = "Voucher already used")]
    VoucherAlreadyUsed,
    /// Error code -100
    #[error(display = "Server internal error")]
    InternalError,
    #[error(display = "Unknown error, _0")]
    UnknownError(i64),
}

impl VoucherError {
    /// Create error from RPC error code.
    pub fn from_rpc_error_code(err_code: i64) -> VoucherError {
        match err_code {
            -400 => VoucherError::BadVoucher,
            -401 => VoucherError::VoucherAlreadyUsed,
            -100 => VoucherError::InternalError,
            err => VoucherError::UnknownError(err),
        }
    }
}
