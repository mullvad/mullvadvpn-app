package net.mullvad.mullvadvpn.lib.billing.model

import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.Purchase

sealed interface PurchaseEvent {
    data object UserCanceled : PurchaseEvent

    data class Error(val exception: BillingException) : PurchaseEvent

    data class PurchaseCompleted(val purchases: List<BillingPurchase>) : PurchaseEvent
}
