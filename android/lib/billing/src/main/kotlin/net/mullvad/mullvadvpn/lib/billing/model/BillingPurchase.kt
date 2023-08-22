package net.mullvad.mullvadvpn.lib.billing.model

data class BillingPurchase(
    val productId: String,
    val token: String
)
