package net.mullvad.mullvadvpn.lib.model

enum class PlayPurchaseVerifyError {
    NoProducts,
    MissingObfuscatedAccountId,
    NoPurchaseToken,
    InvalidPurchase,
    OtherError,
}
