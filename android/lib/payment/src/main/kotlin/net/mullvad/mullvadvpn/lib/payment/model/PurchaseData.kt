package net.mullvad.mullvadvpn.lib.payment.model

data class PurchaseData(
    val token: String,
    val paymentStatus: PaymentStatus?,
)
