package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability

fun PaymentAvailability.toPaymentState(): PaymentState =
    when (this) {
        PaymentAvailability.Error.ServiceUnavailable,
        PaymentAvailability.Error.BillingUnavailable -> PaymentState.Error.Billing
        is PaymentAvailability.Error.Other -> PaymentState.Error.Generic
        is PaymentAvailability.ProductsAvailable -> PaymentState.PaymentAvailable(products)
        PaymentAvailability.ProductsUnavailable -> PaymentState.NoPayment
        PaymentAvailability.NoProductsFound -> PaymentState.NoProductsFounds
        PaymentAvailability.Loading -> PaymentState.Loading
        // Unrecoverable error states
        PaymentAvailability.Error.DeveloperError,
        PaymentAvailability.Error.FeatureNotSupported,
        PaymentAvailability.Error.ItemUnavailable -> PaymentState.NoPayment
    }
