package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.ProductDetailsResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

fun ProductDetailsResult.toPaymentAvailability(
    productIdToPaymentStatus: Map<String, PaymentStatus?>
) =
    when {
        this.billingResult.responseCode == BillingClient.BillingResponseCode.OK &&
            this.productDetailsList.isNullOrEmpty() -> {
            PaymentAvailability.ProductsUnavailable
        }
        this.billingResult.responseCode == BillingClient.BillingResponseCode.OK ->
            PaymentAvailability.ProductsAvailable(
                this.productDetailsList?.toPaymentProducts(productIdToPaymentStatus) ?: emptyList()
            )
        this.billingResult.responseCode == BillingClient.BillingResponseCode.BILLING_UNAVAILABLE ->
            PaymentAvailability.Error.BillingUnavailable
        this.billingResult.responseCode == BillingClient.BillingResponseCode.SERVICE_UNAVAILABLE ->
            PaymentAvailability.Error.ServiceUnavailable
        this.billingResult.responseCode == BillingClient.BillingResponseCode.DEVELOPER_ERROR ->
            PaymentAvailability.Error.DeveloperError
        this.billingResult.responseCode ==
            BillingClient.BillingResponseCode.FEATURE_NOT_SUPPORTED ->
            PaymentAvailability.Error.FeatureNotSupported
        this.billingResult.responseCode == BillingClient.BillingResponseCode.ITEM_UNAVAILABLE ->
            PaymentAvailability.Error.ItemUnavailable
        else ->
            PaymentAvailability.Error.Other(
                BillingException(this.billingResult.responseCode, this.billingResult.debugMessage)
            )
    }
