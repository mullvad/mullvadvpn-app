package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.lib.payment.PaymentRepository

// Just an example:
// * Do not place in root module.
data class PaymentProvider(
    val paymentRepository: PaymentRepository?
)

// Another payment provider option with support of multiple payment options.
typealias PaymentType = String
data class PaymentProviderWithMultiplePaymentTypes(
    val paymentOptions: Map<PaymentType, PaymentRepository>
)
