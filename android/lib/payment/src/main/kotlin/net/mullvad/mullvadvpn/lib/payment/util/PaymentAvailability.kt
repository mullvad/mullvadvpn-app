package net.mullvad.mullvadvpn.lib.payment.util

import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

fun PaymentAvailability?.hasPendingPayment(): Boolean =
    when (this) {
        is PaymentAvailability.ProductsAvailable -> this.products.any { it.status != null }
        else -> false
    }

fun PaymentAvailability.status(): PaymentStatus? =
    when (this) {
        is PaymentAvailability.ProductsAvailable ->
            this.products.firstOrNull { it.status != null }?.status
        else -> null
    }
