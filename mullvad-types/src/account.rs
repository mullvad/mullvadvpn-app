use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};

/// Account identifier used for authentication.
pub type AccountNumber = String;

/// Temporary authorization token derived from a Mullvad account.
pub type AccessToken = String;

/// Account identifier (not used for authentication).
pub type AccountId = String;

/// The payment token returned by initiating a google play purchase.
/// In the API this is called the `obfuscated_id`.
#[cfg(target_os = "android")]
pub type PlayPurchasePaymentToken = String;

/// Account expiration info returned by the API via `/v1/me`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccountData {
    pub id: AccountId,
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
pub struct VoucherSubmission {
    /// Amount of time added to the account
    pub time_added: u64,
    /// Updated expiry time
    pub new_expiry: DateTime<Utc>,
}

/// `PlayPurchase` is provided to google in order to verify that a google play purchase was
/// acknowledged.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg(target_os = "android")]
pub struct PlayPurchase {
    pub product_id: String,
    pub purchase_token: PlayPurchasePaymentToken,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg(target_os = "ios")]
pub struct StorekitTransaction {
    pub transaction: String,
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
