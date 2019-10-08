use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};

pub type AccountToken = String;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccountData {
    pub expiry: DateTime<Utc>,
}

/// Data-structure that's returned from successfuly invocation of the mullvad API's
/// `submit_voucher(account, voucher)` RPC
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct VoucherSubmission {
    /// Amount of time added to the account
    pub time_added: u64,
    /// Updated expiry time
    pub new_expiry: DateTime<Utc>,
}

/// Mapping of mullvad-api errors
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
