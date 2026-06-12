package net.mullvad.mullvadvpn.lib.billing.model

import com.android.billingclient.api.BillingClient.OnPurchasesUpdatedSubResponseCode.NO_APPLICABLE_SUB_RESPONSE_CODE
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PurchasesResult

class BillingException(
    private val responseCode: Int,
    private val subResponseCode: Int = NO_APPLICABLE_SUB_RESPONSE_CODE,
    message: String,
) : Throwable(message) {

    fun toBillingResult(): BillingResult =
        BillingResult.newBuilder()
            .setResponseCode(responseCode)
            .setOnPurchasesUpdatedSubResponseCode(subResponseCode)
            .setDebugMessage(message ?: "")
            .build()

    fun toPurchasesResult(): PurchasesResult = PurchasesResult(toBillingResult(), emptyList())

    override fun toString(): String =
        "BillingException(responseCode=$responseCode, subResponseCode=$subResponseCode, message=${message ?: ""})"
}
