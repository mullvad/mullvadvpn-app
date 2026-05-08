package net.mullvad.mullvadvpn.lib.payment.util

import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability

fun PaymentAvailability?.hasPendingPayment(): Boolean =
    when(this) {
        is PaymentAvailability.ProductsAvailable -> this.products.any { it.status != null }
        else -> false
    }
