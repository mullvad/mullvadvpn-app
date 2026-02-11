package net.mullvad.mullvadvpn.feature.addtime.impl

import net.mullvad.mullvadvpn.feature.addtime.impl.PaymentState.PaymentAvailable
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability

fun PaymentAvailability.toPaymentState(): PaymentState =
    when (this) {
        PaymentAvailability.Error.ServiceUnavailable,
        PaymentAvailability.Error.BillingUnavailable -> PaymentState.Error.Billing
        is PaymentAvailability.Error.Other -> PaymentState.Error.Generic
        is PaymentAvailability.ProductsAvailable -> PaymentAvailable(products)
        PaymentAvailability.ProductsUnavailable -> PaymentState.NoPayment
        PaymentAvailability.NoProductsFound -> PaymentState.NoProductsFounds
        PaymentAvailability.Loading -> PaymentState.Loading
        // Unrecoverable error states
        PaymentAvailability.Error.DeveloperError,
        PaymentAvailability.Error.FeatureNotSupported,
        PaymentAvailability.Error.ItemUnavailable -> PaymentState.NoPayment
    }

fun PaymentAvailability?.hasPendingPayment(): Boolean {
    return this?.let { paymentAvailability ->
        when (val paymentState = paymentAvailability.toPaymentState()) {
            is PaymentAvailable -> paymentState.products.any { it.status != null }
            else -> false
        }
    } == true
}
