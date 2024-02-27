package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.ProductDetailsResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

fun ProductDetailsResult.toPaymentAvailability(
    productIdToPaymentStatus: Map<String, PaymentStatus?>
) =
    when (this.billingResult.responseCode) {
        BillingClient.BillingResponseCode.OK -> {
            val productDetailsList = this.productDetailsList
            if (productDetailsList?.isNotEmpty() == true) {
                PaymentAvailability.ProductsAvailable(
                    productDetailsList.toPaymentProducts(productIdToPaymentStatus)
                )
            } else {
                PaymentAvailability.NoProductsFound
            }
        }
        BillingClient.BillingResponseCode.BILLING_UNAVAILABLE ->
            PaymentAvailability.Error.BillingUnavailable
        BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE ->
            PaymentAvailability.Error.ServiceUnavailable
        BillingClient.BillingResponseCode.DEVELOPER_ERROR ->
            PaymentAvailability.Error.DeveloperError
        BillingClient.BillingResponseCode.FEATURE_NOT_SUPPORTED ->
            PaymentAvailability.Error.FeatureNotSupported
        BillingClient.BillingResponseCode.ITEM_UNAVAILABLE ->
            PaymentAvailability.Error.ItemUnavailable
        else ->
            PaymentAvailability.Error.Other(
                BillingException(this.billingResult.responseCode, this.billingResult.debugMessage)
            )
    }
