package net.mullvad.mullvadvpn.lib.billing.model

import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PurchasesResult

class BillingException(private val responseCode: Int, message: String) : Throwable(message) {

    fun toBillingResult(): BillingResult =
        BillingResult.newBuilder()
            .setResponseCode(responseCode)
            .setDebugMessage(message ?: "")
            .build()

    fun toPurchasesResult(): PurchasesResult =
        PurchasesResult(
            toBillingResult(),
            emptyList()
        )
}
