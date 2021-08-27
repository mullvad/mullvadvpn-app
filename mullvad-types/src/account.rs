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

impl AccountData {
    /// Return true if the account has no time left.
    pub fn is_expired(&self) -> bool {
        self.expiry >= Utc::now()
    }
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
