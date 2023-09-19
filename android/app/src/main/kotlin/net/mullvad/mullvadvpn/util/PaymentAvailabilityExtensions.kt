package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability

fun PaymentAvailability.toPaymentState(): PaymentState =
    when (this) {
        PaymentAvailability.Error.ServiceUnavailable,
        PaymentAvailability.Error.BillingUnavailable -> PaymentState.BillingError
        is PaymentAvailability.Error.Other -> PaymentState.GenericError
        is PaymentAvailability.ProductsAvailable -> PaymentState.PaymentAvailable(products)
        PaymentAvailability.ProductsUnavailable -> PaymentState.NoPayment
        PaymentAvailability.Loading -> PaymentState.Loading
        // Unrecoverable error states
        PaymentAvailability.Error.DeveloperError,
        PaymentAvailability.Error.FeatureNotSupported,
        PaymentAvailability.Error.ItemUnavailable -> PaymentState.NoPayment
    }
