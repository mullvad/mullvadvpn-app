package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetailsResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException

fun ProductDetailsResult.getProductDetails(productId: String): ProductDetails? =
    this.productDetailsList?.firstOrNull { it.productId == productId }

fun ProductDetailsResult.responseCode(): Int = this.billingResult.responseCode

fun ProductDetailsResult.toBillingException(): BillingException =
    BillingException(responseCode = this.responseCode(), message = this.billingResult.debugMessage)
