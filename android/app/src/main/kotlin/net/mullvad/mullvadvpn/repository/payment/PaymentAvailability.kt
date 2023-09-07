package net.mullvad.mullvadvpn.repository.payment

data class PaymentAvailability(
    val webPaymentAvailable: Boolean,
    val billingPaymentAvailability: BillingPaymentAvailability
)
