package net.mullvad.mullvadvpn.lib.payment

data class PaymentAvailability(
    val webPaymentAvailable: Boolean,
    val billingPaymentAvailability: BillingPaymentAvailability
)
