package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException

fun PurchasesResult.purchases(excludePending: Boolean = true): List<Purchase> =
    this.purchasesList.filter {
        if (excludePending) {
            it.purchaseState != Purchase.PurchaseState.PENDING
        } else {
            true
        }
    }

fun PurchasesResult.responseCode(): Int = this.billingResult.responseCode

fun PurchasesResult.toBillingException(): BillingException =
    BillingException(responseCode = this.responseCode(), message = this.billingResult.debugMessage)
