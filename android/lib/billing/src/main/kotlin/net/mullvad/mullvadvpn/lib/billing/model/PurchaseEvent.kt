package net.mullvad.mullvadvpn.lib.billing.model

import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.Purchase

sealed interface PurchaseEvent {
    data object UserCanceled : PurchaseEvent

    data class Error(val result: BillingResult) : PurchaseEvent

    data class PurchaseCompleted(val purchases: List<Purchase>) : PurchaseEvent
}
