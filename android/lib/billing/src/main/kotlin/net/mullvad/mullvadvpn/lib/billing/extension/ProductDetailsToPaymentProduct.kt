package net.mullvad.mullvadvpn.lib.billing.extension

import com.android.billingclient.api.ProductDetails
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice

fun ProductDetails.toPaymentProduct(productIdToStatus: Map<String, PaymentStatus?>) =
    PaymentProduct(
        productId = ProductId(this.productId),
        price = ProductPrice(this.oneTimePurchaseOfferDetails?.formattedPrice ?: ""),
        productIdToStatus[this.productId]
    )

fun List<ProductDetails>.toPaymentProducts(productIdToStatus: Map<String, PaymentStatus?>) =
    this.map { it.toPaymentProduct(productIdToStatus) }
