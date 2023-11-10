package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.Purchase
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

internal fun Int.toPaymentStatus(): PaymentStatus? =
    when (this) {
        Purchase.PurchaseState.PURCHASED -> PaymentStatus.VERIFICATION_IN_PROGRESS
        Purchase.PurchaseState.PENDING -> PaymentStatus.PENDING
        else -> null
    }
