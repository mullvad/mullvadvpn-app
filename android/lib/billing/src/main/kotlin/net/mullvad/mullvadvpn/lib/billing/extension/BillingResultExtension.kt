package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult

internal fun BillingResult.toQueryProductResultError() =
    when (this.responseCode) {
        BillingClient.BillingResponseCode.BILLING_UNAVAILABLE -> QueryProductResult.BillingUnavailable
        BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE -> QueryProductResult.ServiceUnavailable
        BillingClient.BillingResponseCode.ITEM_UNAVAILABLE -> QueryProductResult.ItemUnavailable
        else -> QueryProductResult.Error(BillingException(this.responseCode, this.debugMessage))
    }

internal fun BillingResult.toPurchaseResult() =
    when (this.responseCode) {
        BillingClient.BillingResponseCode.OK -> PurchaseFlowResult.Ok
        BillingClient.BillingResponseCode.USER_CANCELED -> PurchaseFlowResult.UserCancelled
        BillingClient.BillingResponseCode.BILLING_UNAVAILABLE -> PurchaseFlowResult.BillingUnavailable
        BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE -> PurchaseFlowResult.ServiceUnavailable
        BillingClient.BillingResponseCode.ITEM_UNAVAILABLE -> PurchaseFlowResult.ItemUnavailable
        else -> PurchaseFlowResult.Error(BillingException(this.responseCode, this.debugMessage))
    }
