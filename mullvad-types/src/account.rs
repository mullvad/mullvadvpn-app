use chrono::{offset::Utc, DateTime};
#[cfg(target_os = "android")]
use jnix::{FromJava, IntoJava};
use serde::{Deserialize, Serialize};

/// Identifier used to identify a Mullvad account.
pub type AccountToken = String;

/// Identifier used to authenticate a Mullvad account.
pub type AccessToken = String;

// TODO: Should be only android
/// The payment token returned by initiating a google play purchase.
/// In the API this is called the `obfuscated_id`.
pub type PlayPurchaseInitResult = String;

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
        Utc::now() >= self.expiry
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

/// TODO Should be only android
/// `PlayPurchase` is provided to google in order to verify that a google play purchase was acknowledged.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct PlayPurchase {
    pub product_id: String,
    pub purchase_token: String,
}

/// Token used for authentication in the API.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccessTokenData {
    pub access_token: AccessToken,
    pub expiry: DateTime<Utc>,
}

impl AccessTokenData {
    /// Return true if the token is no longer valid.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expiry
    }
}
